use crate::{conf::BucketConfig, Error, Result};

pub use bytes::Bytes;
pub use futures::Stream;

use fstore::{
    http::{Client, ProxyMethod, ProxyResponse, Range},
    Object, RemoveResult,
};
use futures_core::TryStream;
use minty::Uuid;
use std::{error, io, ops::RangeBounds, result};

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

    pub async fn get_object_bytes(&self, id: Uuid) -> Result<Bytes> {
        Ok(self.bucket.get_object_bytes(id).await?)
    }

    pub async fn get_object_stream(
        &self,
        id: Uuid,
        range: impl RangeBounds<u64>,
    ) -> Result<impl Stream<Item = io::Result<Bytes>>> {
        self.bucket
            .get_object_stream_range(id, range)
            .await
            .map_err(|err| match err.kind() {
                fstore::ErrorKind::NotFound => Error::NotFound {
                    entity: "object",
                    id,
                },
                _ => err.into(),
            })
    }

    pub async fn proxy(
        &self,
        object: Uuid,
        method: ProxyMethod,
        range: Option<Range>,
    ) -> Result<ProxyResponse<impl Stream<Item = io::Result<Bytes>>>> {
        self.bucket
            .proxy(object, method, range)
            .await
            .map_err(|err| {
                Error::Internal(format!(
                    "failed to proxy {method} request for object '{object}': \
                    {err}"
                ))
            })
    }

    pub async fn remove_objects(
        &self,
        objects: &[Uuid],
    ) -> Result<RemoveResult> {
        Ok(self.bucket.remove_objects(objects).await?)
    }
}
