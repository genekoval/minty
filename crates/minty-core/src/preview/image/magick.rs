use bytes::{buf::BufMut, Bytes, BytesMut};
use graphicsmagick_sys as gm;
use std::{
    ffi::{c_void, CStr},
    fmt::{self, Display},
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
    result, slice,
};

pub fn initialize() {
    unsafe {
        gm::InitializeMagickEx(
            ptr::null(),
            gm::MAGICK_OPT_NO_SIGNAL_HANDER,
            ptr::null_mut(),
        )
    };
}

pub fn destroy() {
    unsafe { gm::DestroyMagick() };
}

struct ExceptionInfo {
    inner: gm::ExceptionInfo,
}

impl ExceptionInfo {
    fn as_ptr(&mut self) -> *mut gm::ExceptionInfo {
        &mut self.inner
    }
}

impl Default for ExceptionInfo {
    fn default() -> Self {
        let mut exception = gm::ExceptionInfo {
            severity: 0,
            reason: ptr::null_mut(),
            description: ptr::null_mut(),
            error_number: 0,
            module: ptr::null_mut(),
            function: ptr::null_mut(),
            line: 0,
            signature: 0,
        };

        unsafe { gm::GetExceptionInfo(&mut exception) }

        Self { inner: exception }
    }
}

impl Deref for ExceptionInfo {
    type Target = gm::ExceptionInfo;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ExceptionInfo {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Drop for ExceptionInfo {
    fn drop(&mut self) {
        unsafe { gm::DestroyExceptionInfo(&mut self.inner) };
    }
}

#[derive(Debug, thiserror::Error)]
#[error("GraphicsMagick error (severity {severity}): {description}")]
pub struct Exception {
    severity: gm::ExceptionType,
    description: String,
}

impl Exception {
    unsafe fn new(info: ExceptionInfo) -> Self {
        let description = NonNull::new(info.description)
            .map(|description| {
                let cstr = CStr::from_ptr(description.as_ptr());
                let utf8 = String::from_utf8_lossy(cstr.to_bytes());
                utf8.into_owned()
            })
            .unwrap_or_else(|| "Undefined exception".into());

        Self {
            severity: info.severity,
            description,
        }
    }
}

impl From<ExceptionInfo> for Exception {
    fn from(value: ExceptionInfo) -> Self {
        unsafe { Self::new(value) }
    }
}

pub type Result<T> = result::Result<T, Exception>;

#[derive(Clone, Copy, Debug)]
pub struct Geometry {
    pub width: u64,
    pub height: u64,
    pub x: i64,
    pub y: i64,
}

impl Display for Geometry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(width: {}, height: {}, x offset: {}, y offset: {})",
            self.width, self.height, self.x, self.y
        )
    }
}

struct ImageInfo {
    info: NonNull<gm::ImageInfo>,
}

impl ImageInfo {
    pub fn new() -> Self {
        Default::default()
    }

    fn as_ptr(&self) -> *const gm::ImageInfo {
        self.info.as_ptr()
    }
}

impl Default for ImageInfo {
    fn default() -> Self {
        let info = NonNull::new(unsafe { gm::CloneImageInfo(ptr::null()) })
            .expect("CloneImageInfo should return a non-null pointer");

        Self { info }
    }
}

impl Drop for ImageInfo {
    fn drop(&mut self) {
        unsafe { gm::DestroyImageInfo(self.info.as_ptr()) };
    }
}

struct ImageHandle {
    handle: NonNull<gm::Image>,
}

impl ImageHandle {
    fn new(ptr: *mut gm::Image) -> Option<Self> {
        NonNull::new(ptr).map(|handle| Self { handle })
    }

    fn as_ptr(&self) -> *const gm::Image {
        self.handle.as_ptr()
    }

    unsafe fn as_ref(&self) -> &gm::Image {
        self.handle.as_ref()
    }

    unsafe fn as_mut(&mut self) -> &mut gm::Image {
        self.handle.as_mut()
    }
}

impl Drop for ImageHandle {
    fn drop(&mut self) {
        unsafe { gm::DestroyImage(self.handle.as_ptr()) }
    }
}

pub struct Blob {
    data: NonNull<u8>,
    len: usize,
}

impl Blob {
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data.as_ptr(), self.len) }
    }
}

impl Drop for Blob {
    fn drop(&mut self) {
        unsafe { gm::MagickFree(self.data.as_ptr() as *mut c_void) }
    }
}

impl From<Blob> for Bytes {
    fn from(value: Blob) -> Self {
        let bytes = value.as_bytes();
        let mut buf = BytesMut::with_capacity(bytes.len());

        buf.put(bytes);
        buf.freeze()
    }
}

pub struct Image {
    info: ImageInfo,
    handle: ImageHandle,
}

impl Image {
    pub fn from_bytes(bytes: Bytes) -> Result<Self> {
        let info = ImageInfo::new();
        let mut exception = ExceptionInfo::default();

        let image = unsafe {
            gm::BlobToImage(
                info.as_ptr(),
                bytes.as_ptr().cast(),
                bytes.len() as u64,
                exception.as_ptr(),
            )
        };

        let handle = ImageHandle::new(image).ok_or(exception)?;

        Ok(Self { info, handle })
    }

    pub fn from_raw_pixels(
        width: u64,
        height: u64,
        pixels: *const u8,
    ) -> Result<Self> {
        let mut exception = ExceptionInfo::default();

        let image = unsafe {
            gm::ConstituteImage(
                width,
                height,
                c"RGB".as_ptr(),
                gm::StorageType_CharPixel,
                pixels.cast(),
                exception.as_ptr(),
            )
        };

        let handle = ImageHandle::new(image).ok_or(exception)?;

        Ok(Self {
            info: ImageInfo::new(),
            handle,
        })
    }

    pub fn width(&self) -> u64 {
        self.handle().columns
    }

    pub fn height(&self) -> u64 {
        self.handle().rows
    }

    pub fn crop(&mut self, geometry: Geometry) -> Result<()> {
        let mut exception = ExceptionInfo::default();
        let geometry = gm::RectangleInfo {
            width: geometry.width,
            height: geometry.height,
            x: geometry.x,
            y: geometry.y,
        };

        let image = unsafe {
            gm::CropImage(self.handle.as_ptr(), &geometry, exception.as_ptr())
        };

        self.handle = ImageHandle::new(image).ok_or(exception)?;
        Ok(())
    }

    pub fn thumbnail(&mut self, width: u64, height: u64) -> Result<()> {
        let mut exception = ExceptionInfo::default();

        let image = unsafe {
            gm::ThumbnailImage(
                self.handle.as_ptr(),
                width,
                height,
                exception.as_ptr(),
            )
        };

        self.handle = ImageHandle::new(image).ok_or(exception)?;
        Ok(())
    }

    pub fn magick(&mut self, format: &str) {
        let magick = self.handle_mut().magick.as_mut_ptr();

        unsafe {
            format.as_ptr().copy_to(magick as *mut u8, format.len());
            magick.add(format.len()).write('\0' as i8);
        }
    }

    pub fn write(&mut self) -> Result<Blob> {
        let mut exception = ExceptionInfo::default();
        let mut len: u64 = 0;

        let data = unsafe {
            gm::ImageToBlob(
                self.info.as_ptr(),
                self.handle.as_mut(),
                &mut len,
                exception.as_ptr(),
            )
        };

        let data = NonNull::new(data as *mut u8).ok_or(exception)?;

        Ok(Blob {
            data,
            len: len as usize,
        })
    }

    pub fn bytes(&mut self) -> Result<Bytes> {
        Ok(self.write()?.into())
    }

    fn handle(&self) -> &gm::Image {
        unsafe { self.handle.as_ref() }
    }

    fn handle_mut(&mut self) -> &mut gm::Image {
        unsafe { self.handle.as_mut() }
    }
}
