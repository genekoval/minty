use super::{
    error::{ToError, ToResult},
    ffmpeg::{
        self, AVCodec, AVFormatContext, AVMediaType, AVStream,
        AVERROR_DECODER_NOT_FOUND, AVERROR_EOF, AVERROR_STREAM_NOT_FOUND,
    },
    io::IoContext,
    packet::*,
};

use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr,
};

pub struct FormatContext<'a> {
    handle: *mut AVFormatContext,
    _phantom: PhantomData<&'a IoContext>,
}

impl<'a> FormatContext<'a> {
    pub fn new(io: &'a IoContext) -> Result<Self, String> {
        let handle = unsafe {
            ffmpeg::avformat_alloc_context().as_mut().ok_or_else(|| {
                String::from("failed to allocate format context")
            })?
        };

        handle.pb = io.as_ptr();

        unsafe {
            ffmpeg::avformat_open_input(
                &mut (handle as *mut AVFormatContext),
                ptr::null(),
                ptr::null(),
                ptr::null_mut(),
            )
            .to_result()
            .map_err(|err| {
                ffmpeg::avformat_free_context(handle);
                format!("failed to open input: {err}")
            })?;

            ffmpeg::avformat_find_stream_info(handle, ptr::null_mut())
                .to_result()
                .map_err(|err| format!("failed to find stream info: {err}"))?;
        }

        Ok(Self {
            handle,
            _phantom: PhantomData,
        })
    }

    pub fn find_best_stream(
        &self,
        ty: AVMediaType,
    ) -> Result<Option<(&AVStream, &AVCodec)>, String> {
        let mut decoder: *const AVCodec = ptr::null_mut();

        match unsafe {
            ffmpeg::av_find_best_stream(
                self.handle,
                ty,
                -1,
                -1,
                &mut decoder,
                0,
            )
        } {
            AVERROR_STREAM_NOT_FOUND => Ok(None),
            AVERROR_DECODER_NOT_FOUND => Err("unsupported codec".into()),
            i => unsafe {
                let entry = *self.streams.add(i as usize);
                let stream = entry.as_ref().expect("stream should not be null");
                let decoder =
                    decoder.as_ref().expect("decoder should not be null");
                Ok(Some((stream, decoder)))
            },
        }
    }

    pub fn read_frame<'p>(
        &self,
        packet: &'p mut PacketHandle,
    ) -> Result<Option<Packet<'p>>, String> {
        match unsafe { ffmpeg::av_read_frame(self.handle, packet.as_mut_ptr()) }
        {
            0 => Ok(Some(Packet::new(packet))),
            AVERROR_EOF => Ok(None),
            result => {
                Err(format!("error reading frame: {}", result.to_error()))
            }
        }
    }
}

impl<'a> Deref for FormatContext<'a> {
    type Target = AVFormatContext;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.handle }
    }
}

impl<'a> DerefMut for FormatContext<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.handle }
    }
}

impl<'a> Drop for FormatContext<'a> {
    fn drop(&mut self) {
        unsafe { ffmpeg::avformat_close_input(&mut self.handle) };
    }
}
