use crate::{Error, Result};

use elasticsearch::http::response::Response;

pub trait ResponseExt: Sized {
    async fn check(self) -> Result<Self>;
}

impl ResponseExt for Response {
    async fn check(self) -> Result<Self> {
        let status = self.status_code();

        if status.is_client_error() || status.is_server_error() {
            let err = self.text().await?;
            Err(Error::Internal(format!("Elasticsearch error: {err}")))
        } else {
            Ok(self)
        }
    }
}
