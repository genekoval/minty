use super::Repo;

use crate::{db::Id, preview, search::Index, Error, Result};

use futures::{stream::BoxStream, StreamExt};
use log::error;
use serde::Serialize;
use std::{result, sync::Arc};
use tokio::sync::Semaphore;
use tokio_util::task::TaskTracker;

#[derive(Clone)]
pub struct Task {
    repo: Arc<Repo>,
    task: crate::Task,
}

impl Task {
    pub fn new(repo: &Arc<Repo>, task: crate::Task) -> Self {
        Self {
            repo: repo.clone(),
            task,
        }
    }

    pub async fn regenerate_previews(
        &self,
        batch_size: usize,
        max_tasks: usize,
    ) -> Result<()> {
        let tracker = TaskTracker::new();
        let semaphore = Arc::new(Semaphore::new(max_tasks));
        let mut error: Option<Error> = None;
        let mut stream = self.repo.database.stream_objects().chunks(batch_size);

        'stream: while let Some(chunk) = stream.next().await {
            let objects = match chunk
                .into_iter()
                .collect::<result::Result<Vec<_>, _>>()
                .map(|objects| {
                    objects
                        .into_iter()
                        .map(|object| object.id)
                        .collect::<Vec<_>>()
                }) {
                Ok(objects) => objects,
                Err(err) => {
                    error = Some(err.into());
                    break 'stream;
                }
            };

            let objects = match self.repo.bucket.get_objects(&objects).await {
                Ok(objects) => objects,
                Err(err) => {
                    error = Some(err);
                    break 'stream;
                }
            };

            for object in objects {
                let permit = tokio::select! {
                    biased;

                    _ = self.task.cancelled() => {
                        break 'stream;
                    }
                    permit = semaphore.clone().acquire_owned() => {
                        permit.unwrap()
                    }
                };

                let task = self.clone();

                tracker.spawn(async move {
                    task.regenerate_previews_subtask(&object).await;
                    drop(permit);
                });
            }
        }

        tracker.close();
        tracker.wait().await;

        match error {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }

    async fn regenerate_previews_subtask(&self, object: &fstore::Object) {
        match preview::generate_preview(&self.repo.bucket, object).await {
            Ok(preview) => {
                if let Err(err) = self
                    .repo
                    .database
                    .update_object_preview(object.id, preview)
                    .await
                {
                    error!(
                        "Failed to update object \
                        preview for {}: {err}",
                        object.id
                    );
                    self.task.error();
                }
            }
            Err(message) => {
                if let Err(err) = self
                    .repo
                    .database
                    .create_object_preview_error(object.id, &message)
                    .await
                {
                    error!(
                        "Failed to write object preview error \
                        for {}: {err}; preview error: {message}",
                        object.id
                    );
                }
                self.task.error();
            }
        }

        self.task.increment();
    }

    pub async fn reindex<T>(
        &self,
        index: &Index,
        batch_size: usize,
        stream: BoxStream<'_, sqlx::Result<T>>,
    ) -> Result<()>
    where
        T: Id + Serialize,
    {
        index.recreate().await?;

        let mut stream = stream.chunks(batch_size);

        while let Some(chunk) = stream.next().await {
            let items =
                chunk.into_iter().collect::<result::Result<Vec<_>, _>>()?;

            index.bulk_create(&items).await?;
            self.task.progress(items.len());
        }

        index.refresh().await?;

        Ok(())
    }
}
