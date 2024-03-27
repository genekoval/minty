use super::{
    error::ToResult,
    ffmpeg::{
        self, AVCodec, AVCodecContext, AVCodecParameters, AVPixelFormat,
        AVERROR, AVERROR_EOF, AVERROR_INPUT_CHANGED, EAGAIN, EINVAL,
    },
    frame::Frame,
    packet::Packet,
};

use std::ptr::{self, NonNull};

pub struct CodecContext<'a> {
    codec: &'a AVCodec,
    ctx: NonNull<AVCodecContext>,
}

impl<'a> CodecContext<'a> {
    pub fn new(codec: &'a AVCodec) -> Result<Self, String> {
        let ctx =
            NonNull::new(unsafe { ffmpeg::avcodec_alloc_context3(codec) })
                .ok_or_else(|| {
                    String::from("could not allocate video codec context")
                })?;

        Ok(Self { codec, ctx })
    }

    pub fn as_ptr(&self) -> *mut AVCodecContext {
        self.ctx.as_ptr()
    }

    pub fn open(&self) -> Result<(), String> {
        unsafe {
            ffmpeg::avcodec_open2(self.as_ptr(), self.codec, ptr::null_mut())
                .to_result()
                .map_err(|err| format!("could not open codec: {err}"))?
        };

        Ok(())
    }

    pub fn copy_params(
        &self,
        params: *const AVCodecParameters,
    ) -> Result<(), String> {
        let result = unsafe {
            ffmpeg::avcodec_parameters_to_context(self.as_ptr(), params)
        };

        if result < 0 {
            Err("failed to copy codec parameters to context".into())
        } else {
            Ok(())
        }
    }

    pub fn decode(
        &self,
        packet: &Packet,
        frame: &mut Frame,
    ) -> Result<bool, String> {
        unsafe {
            ffmpeg::avcodec_send_packet(self.as_ptr(), packet.as_ptr())
                .to_result()
                .map_err(|err| {
                    format!("error sending packet for decoding: {err}")
                })?
        };

        let result = unsafe {
            ffmpeg::avcodec_receive_frame(self.as_ptr(), frame.as_mut_ptr())
        };

        if result == 0 {
            return Ok(true);
        }

        if result == AVERROR(EAGAIN) {
            return Ok(false);
        }

        let reason = match result {
            AVERROR_EOF => "the decoder has been fully flushed, \
                         and there will be no more output frames",
            AVERROR_INPUT_CHANGED => "current decoded frame has changed \
                                   parameters with respect to first decoded frame",
            result if result == AVERROR(EINVAL) => "codec not opened, \
                              or it is an encoder",
            _ => "legitimate decoding errors",
        };

        Err(format!("error during decoding: {reason}"))
    }

    pub fn width(&self) -> i32 {
        unsafe { self.ctx.as_ref().width }
    }

    pub fn height(&self) -> i32 {
        unsafe { self.ctx.as_ref().height }
    }

    pub fn pixel_format(&self) -> AVPixelFormat {
        unsafe { self.ctx.as_ref().pix_fmt }
    }
}

impl<'a> Drop for CodecContext<'a> {
    fn drop(&mut self) {
        unsafe { ffmpeg::avcodec_free_context(&mut self.ctx.as_ptr()) };
    }
}
