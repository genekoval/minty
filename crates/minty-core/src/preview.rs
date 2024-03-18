mod image;

use crate::obj::Bucket;

use fstore::Object;
use minty::Uuid;
use std::result;

pub type Result = result::Result<Option<Uuid>, String>;

pub struct Env {
    #[allow(dead_code)]
    image: image::Env,
}

impl Env {
    pub fn initialize() -> Self {
        Self {
            image: image::Env::initialize(),
        }
    }
}

pub async fn generate_preview(bucket: &Bucket, object: &Object) -> Result {
    match object.r#type.as_str() {
        "image" => image::generate_preview(bucket, object).await,
        _ => Ok(None),
    }
}
