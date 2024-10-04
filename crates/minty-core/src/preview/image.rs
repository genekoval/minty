mod magick;

pub use magick::*;

use super::{Bucket, Object, Result};

use bytes::Bytes;
use log::debug;
use std::{cmp, result};
use tokio::task;

const THUMBNAIL_FORMAT: &str = "JPEG";
const THUMBNAIL_SIZE: u64 = 250;

pub struct Env;

impl Env {
    pub fn initialize() -> Self {
        magick::initialize();
        debug!("Image environment initialized");

        Self
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        magick::destroy();
        debug!("Image environment destroyed");
    }
}

pub fn make_thumbnail(image: &mut Image) -> result::Result<Bytes, String> {
    let width = image.width();
    let height = image.height();

    debug!("Image dimensions: {width} x {height}");

    // Crop the image to a square if it is not a square already
    if width != height {
        let smaller = cmp::min(width, height);
        let larger = cmp::max(width, height);
        let offset = ((larger - smaller) / 2) as i64;

        let crop = Geometry {
            width: smaller,
            height: smaller,
            x: if smaller == height { offset } else { 0 },
            y: if smaller == width { offset } else { 0 },
        };

        debug!("Crop image to {crop}");

        image
            .crop(crop)
            .map_err(|err| format!("failed to crop image to {crop}: {err}"))?;
    }

    image
        .thumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE)
        .map_err(|err| format!("failed to resize image: {err}"))?;

    image.magick(THUMBNAIL_FORMAT);

    let thumbnail = image.bytes().map_err(|err| {
        format!("failed to write image data as a {THUMBNAIL_FORMAT}: {err}")
    })?;

    Ok(thumbnail)
}

pub async fn generate_preview(bucket: &Bucket, object: &Object) -> Result {
    let bytes = bucket
        .get_object_bytes(object.id)
        .await
        .map_err(|err| format!("failed to retrieve image data: {err}"))?;

    let thumbnail = task::spawn_blocking(move || {
        let mut image = Image::from_bytes(bytes)
            .map_err(|err| format!("failed to read image data: {err}"))?;

        make_thumbnail(&mut image)
    })
    .await
    .map_err(|err| err.to_string())??;

    let preview = bucket.add_object(thumbnail).await.map_err(|err| {
        format!("failed to upload thumbnail to bucket: {err}")
    })?;

    Ok(Some(preview.id))
}
