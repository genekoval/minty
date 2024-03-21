use crate::{conf::BucketConfig, db, Result};

use bytes::Bytes;
use fstore::{http::Client, Object, RemoveResult};
use futures::Stream;
use futures_core::TryStream;
use minty::{ObjectPreview, ObjectSummary, Uuid};
use std::{error, io, result};

#[derive(Clone, Debug)]
pub struct Bucket {
    bucket: fstore::http::Bucket,
}

impl Bucket {
    pub async fn new(
        BucketConfig { url, bucket }: &BucketConfig,
    ) -> result::Result<Self, String> {
        let client = Client::new(url);
        let (bucket, _) = client
            .get_bucket(bucket)
            .await
            .map_err(|err| format!("failed to retrieve bucket info: {err}"))?;

        Ok(Self { bucket })
    }

    pub async fn add_object(&self, bytes: Bytes) -> Result<Object> {
        Ok(self.bucket.add_object_bytes(bytes).await?)
    }

    pub async fn add_object_stream<S>(&self, stream: S) -> Result<Object>
    where
        S: TryStream + Send + Sync + 'static,
        S::Error: Into<Box<dyn error::Error + Send + Sync>>,
        Bytes: From<S::Ok>,
    {
        Ok(self.bucket.add_object_stream(stream).await?)
    }

    pub async fn get_object(&self, id: Uuid) -> Result<fstore::Object> {
        Ok(self.bucket.get_object(id).await?)
    }

    pub async fn get_objects(
        &self,
        objects: &[Uuid],
    ) -> Result<Vec<fstore::Object>> {
        Ok(self.bucket.get_objects(objects).await?)
    }

    pub async fn get_object_bytes(
        &self,
        id: Uuid,
    ) -> Result<(ObjectSummary, Bytes)> {
        let (summary, bytes) = self.bucket.get_object_bytes(id).await?;
        let summary = ObjectSummary {
            media_type: summary.media_type,
            size: summary.size,
        };

        Ok((summary, bytes))
    }

    pub async fn get_object_stream(
        &self,
        id: Uuid,
    ) -> Result<(ObjectSummary, impl Stream<Item = io::Result<Bytes>>)> {
        let (summary, stream) = self.bucket.get_object_stream(id).await?;
        let summary = ObjectSummary {
            media_type: summary.media_type,
            size: summary.size,
        };

        Ok((summary, stream))
    }

    pub async fn get_object_previews(
        &self,
        objects: Vec<db::Object>,
    ) -> Result<Vec<ObjectPreview>> {
        let mut result: Vec<ObjectPreview> = Vec::with_capacity(objects.len());

        for object in objects {
            let metadata = self.bucket.get_object(object.id).await?;

            result.push(ObjectPreview {
                id: object.id,
                preview_id: object.preview_id,
                r#type: metadata.r#type,
                subtype: metadata.subtype,
            });
        }

        Ok(result)
    }

    pub async fn remove_objects(
        &self,
        objects: &[Uuid],
    ) -> Result<RemoveResult> {
        Ok(self.bucket.remove_objects(objects).await?)
    }
}
