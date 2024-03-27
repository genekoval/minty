use super::{
    codec::CodecContext,
    ffmpeg::{self, AVPixelFormat, SWS_BILINEAR},
    frame::Frame,
};

use std::ptr::{self, NonNull};

pub struct SwsContext {
    ctx: NonNull<ffmpeg::SwsContext>,
    height: i32,
}

impl SwsContext {
    pub fn new(
        codec: &CodecContext,
        pixel_format: AVPixelFormat,
    ) -> Result<Self, String> {
        let width = codec.width();
        let height = codec.height();

        let ctx = NonNull::new(unsafe {
            ffmpeg::sws_getContext(
                width,
                height,
                codec.pixel_format(),
                width,
                height,
                pixel_format,
                SWS_BILINEAR,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null(),
            )
        })
        .ok_or_else(|| String::from("failed to allocate scaling context"))?;

        Ok(Self { ctx, height })
    }

    pub fn scale(&self, src: &Frame, dest: &Frame) {
        unsafe {
            ffmpeg::sws_scale(
                self.ctx.as_ptr(),
                src.data.map(|ptr| ptr as *const _).as_ptr(),
                src.linesize.as_ptr(),
                0,
                self.height,
                dest.data.as_ptr(),
                dest.linesize.as_ptr(),
            )
        };
    }
}

impl Drop for SwsContext {
    fn drop(&mut self) {
        unsafe { ffmpeg::sws_freeContext(self.ctx.as_ptr()) };
    }
}
