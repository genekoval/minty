use super::{
    video::{FormatContext, IoContext, PacketHandle},
    Bucket, Object, Result,
};

use bytes::Bytes;
use ffmpeg_sys_next::AVMediaType::AVMEDIA_TYPE_VIDEO;
use log::debug;
use std::result;
use tokio::task;

fn find_embedded_image(audio: Bytes) -> result::Result<Option<Bytes>, String> {
    let io = IoContext::new(audio.as_ref())?;
    let format = FormatContext::new(&io)?;

    let Some((stream, _)) = format.find_best_stream(AVMEDIA_TYPE_VIDEO)? else {
        debug!("Audio file does not contain a video stream");
        return Ok(None);
    };

    let mut packet = PacketHandle::new()?;

    while let Some(packet) = format.read_frame(&mut packet)? {
        if !packet.is_stream(stream) {
            continue;
        }

        let image = Bytes::copy_from_slice(packet.data());
        return Ok(Some(image));
    }

    Ok(None)
}

pub async fn generate_preview(bucket: &Bucket, object: &Object) -> Result {
    let (_, bytes) = bucket
        .get_object_bytes(object.id)
        .await
        .map_err(|err| format!("failed to retrieve audio data: {err}"))?;

    let Some(image) = task::spawn_blocking(move || find_embedded_image(bytes))
        .await
        .map_err(|err| err.to_string())??
    else {
        return Ok(None);
    };

    let object = bucket.add_object(image).await.map_err(|err| {
        format!("failed to upload audio preview to bucket: {err}")
    })?;

    Ok(Some(object.id))
}
