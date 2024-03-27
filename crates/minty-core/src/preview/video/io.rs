use super::ffmpeg::{
    self, AVIOContext, AVERROR_EOF, AVSEEK_FORCE, AVSEEK_SIZE, SEEK_CUR,
    SEEK_END, SEEK_SET,
};
use std::{cmp, ffi::c_void, ops::Range, pin::Pin, ptr};

const BUFFER_SIZE: usize = 4096;

unsafe extern "C" fn read_packet(
    opaque: *mut c_void,
    buffer: *mut u8,
    buffer_size: i32,
) -> i32 {
    let bd = opaque.cast::<BufferData>().as_mut().unwrap();

    let size = cmp::min(buffer_size, bd.remaining());
    if size == 0 {
        return AVERROR_EOF;
    }

    bd.copy_to(buffer, size);

    size
}

unsafe extern "C" fn seek(
    opaque: *mut c_void,
    offset: i64,
    mut whence: i32,
) -> i64 {
    let bd = opaque.cast::<BufferData>().as_mut().unwrap();

    let size = whence & AVSEEK_SIZE;
    if size == AVSEEK_SIZE {
        return bd.len();
    }

    whence &= !AVSEEK_FORCE;
    whence &= !AVSEEK_SIZE;

    if whence == SEEK_SET {
        bd.seek_start(offset);
    } else if whence == SEEK_CUR {
        bd.seek_cursor(offset);
    } else if whence == SEEK_END {
        bd.seek_end(offset);
    }

    bd.cursor_position()
}

pub struct Buffer(*mut u8);

impl Buffer {
    pub fn new(size: usize) -> Result<Self, String> {
        let handle = unsafe { ffmpeg::av_malloc(size).cast::<u8>() };

        if handle.is_null() {
            Err("failed to allocate buffer".into())
        } else {
            Ok(Self(handle))
        }
    }

    pub fn as_ptr(&self) -> *mut u8 {
        self.0
    }

    pub fn release(mut self) -> *mut u8 {
        let handle = self.0;
        self.0 = ptr::null_mut();
        handle
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe { ffmpeg::av_free(self.0.cast()) };
    }
}

struct BufferData {
    range: Range<*const u8>,
    cursor: *const u8,
}

impl BufferData {
    fn new(data: &[u8]) -> Self {
        Self {
            range: data.as_ptr_range(),
            cursor: data.as_ptr(),
        }
    }

    unsafe fn copy_to(&mut self, dest: *mut u8, count: i32) {
        let count = count as usize;

        self.cursor.copy_to(dest, count);
        self.cursor = self.cursor.add(count);
    }

    unsafe fn len(&self) -> i64 {
        self.range.end.offset_from(self.range.start) as i64
    }

    unsafe fn cursor_position(&self) -> i64 {
        self.cursor.offset_from(self.range.start) as i64
    }

    unsafe fn remaining(&self) -> i32 {
        self.range.end.offset_from(self.cursor) as i32
    }

    unsafe fn seek(&mut self, p: *const u8, offset: i64) {
        self.cursor = p.offset(offset as isize);
    }

    unsafe fn seek_start(&mut self, offset: i64) {
        self.seek(self.range.start, offset);
    }

    unsafe fn seek_end(&mut self, offset: i64) {
        self.seek(self.range.end, offset);
    }

    unsafe fn seek_cursor(&mut self, offset: i64) {
        self.seek(self.cursor, offset);
    }
}

pub struct IoContext {
    ctx: *mut AVIOContext,
    _data: Pin<Box<BufferData>>,
}

impl IoContext {
    pub fn new(data: &[u8]) -> Result<Self, String> {
        let mut data = Box::pin(BufferData::new(data));
        let data_ptr: *mut BufferData = &mut *data;

        let buffer = Buffer::new(BUFFER_SIZE)?;

        let ctx = unsafe {
            ffmpeg::avio_alloc_context(
                buffer.as_ptr(),
                BUFFER_SIZE as i32,
                0,
                data_ptr.cast(),
                Some(read_packet),
                None,
                Some(seek),
            )
        };

        if ctx.is_null() {
            return Err(String::from("failed to allocate IO context"));
        }

        buffer.release();

        Ok(Self { ctx, _data: data })
    }

    pub fn as_ptr(&self) -> *mut AVIOContext {
        self.ctx
    }
}

impl Drop for IoContext {
    fn drop(&mut self) {
        unsafe {
            let ctx = &mut *self.ctx;
            let buffer: *mut *mut u8 = &mut ctx.buffer;

            ffmpeg::av_freep(buffer.cast());
            ffmpeg::avio_context_free(&mut self.ctx);
        }
    }
}
