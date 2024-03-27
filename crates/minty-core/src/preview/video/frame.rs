use super::{
    error::ToResult,
    ffmpeg::{self, AVFrame, AVPixelFormat},
};

use std::ops::{Deref, DerefMut};

pub struct Frame(*mut AVFrame);

impl Frame {
    pub fn new() -> Result<Self, String> {
        let handle = unsafe { ffmpeg::av_frame_alloc() };

        if handle.is_null() {
            Err("failed to allocate frame".into())
        } else {
            Ok(Self(handle))
        }
    }

    pub fn as_mut_ptr(&mut self) -> *mut AVFrame {
        self.0
    }

    pub fn is_key_frame(&self) -> bool {
        self.key_frame > 0
    }
}

impl Deref for Frame {
    type Target = AVFrame;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref().expect("AVFrame pointer should not be null") }
    }
}

impl DerefMut for Frame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut().expect("AVFrame pointer should not be null") }
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        unsafe { ffmpeg::av_frame_free(&mut self.0) };
    }
}

pub struct ImageFrame(Frame);

impl ImageFrame {
    pub fn new(
        width: i32,
        height: i32,
        format: AVPixelFormat,
        align: i32,
    ) -> Result<Self, String> {
        let mut frame = Frame::new()?;

        unsafe {
            ffmpeg::av_image_alloc(
                frame.data.as_mut_ptr(),
                frame.linesize.as_mut_ptr(),
                width,
                height,
                format,
                align,
            )
            .to_result()
            .map_err(|err| format!("failed to allocate image: {err}"))?;
        }

        Ok(Self(frame))
    }

    pub fn frame(&self) -> &Frame {
        &self.0
    }

    pub fn pixels(&self) -> *mut u8 {
        unsafe { *self.0.data.get_unchecked(0) }
    }
}

impl Drop for ImageFrame {
    fn drop(&mut self) {
        let p: *mut *mut _ = &mut self.pixels();
        unsafe { ffmpeg::av_freep(p.cast()) };
    }
}
