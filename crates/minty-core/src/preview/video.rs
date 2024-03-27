mod codec;
mod error;
mod format;
mod frame;
mod io;
mod packet;
mod sws;

use codec::CodecContext;
use format::FormatContext;
use frame::*;
use io::IoContext;
use packet::PacketHandle;
use sws::SwsContext;

use super::{
    image::{self, Image},
    Bucket, Object, Result,
};

use bytes::Bytes;
use ffmpeg_sys_next::{
    self as ffmpeg, AVMediaType::AVMEDIA_TYPE_VIDEO, AVPixelFormat,
};
use std::result;
use tokio::task;

fn make_image(
    codec: &CodecContext,
    video_frame: &Frame,
) -> result::Result<Bytes, String> {
    const PIXEL_FORMAT: AVPixelFormat = AVPixelFormat::AV_PIX_FMT_RGB24;

    let image_frame =
        ImageFrame::new(codec.width(), codec.height(), PIXEL_FORMAT, 1)?;

    SwsContext::new(codec, PIXEL_FORMAT)?
        .scale(video_frame, image_frame.frame());

    let mut image = Image::from_raw_pixels(
        codec.width() as u64,
        codec.height() as u64,
        image_frame.pixels(),
    )
    .map_err(|err| format!("failed to create image from video frame: {err}"))?;

    image::make_thumbnail(&mut image)
}

fn find_preview_frame(video: Bytes) -> result::Result<Bytes, String> {
    let io = IoContext::new(video.as_ref())?;
    let format = FormatContext::new(&io)?;

    let (stream, codec) = format
        .find_best_stream(AVMEDIA_TYPE_VIDEO)?
        .ok_or_else(|| String::from("no video stream found"))?;

    let codec = CodecContext::new(codec)?;
    codec.copy_params(stream.codecpar)?;
    codec.open()?;

    let mut packet = PacketHandle::new()?;
    let mut frame = Frame::new()?;

    while let Some(packet) = format.read_frame(&mut packet)? {
        if !packet.is_stream(stream) {
            continue;
        }

        if codec.decode(&packet, &mut frame)? && frame.is_key_frame() {
            return make_image(&codec, &frame);
        }
    }

    Err("failed to obtain video frame for preview".into())
}

pub async fn generate_preview(bucket: &Bucket, object: &Object) -> Result {
    let (_, bytes) = bucket
        .get_object_bytes(object.id)
        .await
        .map_err(|err| format!("failed to retrieve video data: {err}"))?;

    let image = task::spawn_blocking(move || find_preview_frame(bytes))
        .await
        .map_err(|err| err.to_string())??;

    let object = bucket.add_object(image).await.map_err(|err| {
        format!("failed to upload video thumbnail to bucket: {err}")
    })?;

    Ok(Some(object.id))
}
