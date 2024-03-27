use super::ffmpeg::{self, AVPacket, AVStream};

use std::ops::Deref;

pub struct PacketHandle(*mut AVPacket);

impl PacketHandle {
    pub fn new() -> Result<Self, String> {
        let handle = unsafe { ffmpeg::av_packet_alloc() };

        if handle.is_null() {
            Err("failed to allocate packet".into())
        } else {
            Ok(Self(handle))
        }
    }

    pub fn as_ptr(&self) -> *const AVPacket {
        self.0
    }

    pub fn as_mut_ptr(&mut self) -> *mut AVPacket {
        self.0
    }

    pub fn unref(&mut self) {
        unsafe { ffmpeg::av_packet_unref(self.0) };
    }
}

impl Deref for PacketHandle {
    type Target = AVPacket;

    fn deref(&self) -> &Self::Target {
        unsafe {
            self.0
                .as_ref()
                .expect("AVPacket pointer should not be null")
        }
    }
}

impl Drop for PacketHandle {
    fn drop(&mut self) {
        unsafe { ffmpeg::av_packet_free(&mut self.0) };
    }
}

pub struct Packet<'a>(&'a mut PacketHandle);

impl<'a> Packet<'a> {
    pub fn new(handle: &'a mut PacketHandle) -> Self {
        Self(handle)
    }

    pub fn as_ptr(&self) -> *const AVPacket {
        self.0.as_ptr()
    }

    pub fn is_stream(&self, stream: &AVStream) -> bool {
        self.0.stream_index == stream.index
    }
}

impl<'a> Drop for Packet<'a> {
    fn drop(&mut self) {
        self.0.unref();
    }
}
