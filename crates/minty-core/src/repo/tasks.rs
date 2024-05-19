use super::Repo;

use crate::{Result, Task};

use std::sync::Arc;
use tokio::task::{self, JoinHandle};

pub struct Tasks<'a> {
    repo: &'a Arc<Repo>,
}

impl<'a> Tasks<'a> {
    pub(super) fn new(repo: &'a Arc<Repo>) -> Self {
        Self { repo }
    }

    pub async fn regenerate_previews(
        &self,
        batch_size: usize,
        max_tasks: usize,
    ) -> Result<(Task, JoinHandle<Result<()>>)> {
        let total = self
            .repo
            .database
            .read_object_total()
            .await?
            .try_into()
            .unwrap();

        let task = Task::new(total);
        let guard = task.guard();
        let repo = self.repo.clone();

        let handle = task::spawn(async move {
            repo.task(guard.task())
                .regenerate_previews(batch_size, max_tasks)
                .await
        });

        Ok((task, handle))
    }

    pub async fn reindex_posts(
        &self,
        batch_size: usize,
    ) -> Result<(Task, JoinHandle<Result<()>>)> {
        let total = self
            .repo
            .database
            .read_post_total()
            .await?
            .try_into()
            .unwrap();

        let task = Task::new(total);
        let guard = task.guard();
        let repo = self.repo.clone();

        let handle = task::spawn(async move {
            let index = &repo.search.indices.post;
            let stream = repo.database.read_post_search();

            repo.task(guard.task())
                .reindex(index, batch_size, stream)
                .await
        });

        Ok((task, handle))
    }

    pub async fn reindex_tags(
        &self,
        batch_size: usize,
    ) -> Result<(Task, JoinHandle<Result<()>>)> {
        let total = self
            .repo
            .database
            .read_tag_total()
            .await?
            .try_into()
            .unwrap();

        let task = Task::new(total);
        let guard = task.guard();
        let repo = self.repo.clone();

        let handle = task::spawn(async move {
            let index = &repo.search.indices.tag;
            let stream = repo.database.read_tag_search();

            repo.task(guard.task())
                .reindex(index, batch_size, stream)
                .await
        });

        Ok((task, handle))
    }

    pub async fn reindex_users(
        &self,
        batch_size: usize,
    ) -> Result<(Task, JoinHandle<Result<()>>)> {
        let total = self
            .repo
            .database
            .read_user_total()
            .await?
            .try_into()
            .unwrap();

        let task = Task::new(total);
        let guard = task.guard();
        let repo = self.repo.clone();

        let handle = task::spawn(async move {
            let index = &repo.search.indices.user;
            let stream = repo.database.read_user_search();

            repo.task(guard.task())
                .reindex(index, batch_size, stream)
                .await
        });

        Ok((task, handle))
    }
}
