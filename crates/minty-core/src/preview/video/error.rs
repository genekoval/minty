use super::ffmpeg::{self, AV_ERROR_MAX_STRING_SIZE};

use std::ffi::CStr;

pub trait ToError {
    fn to_error(self) -> String;
}

pub trait ToResult
where
    Self: Sized,
{
    fn to_result(self) -> Result<Self, String>;
}

impl ToError for i32 {
    fn to_error(self) -> String {
        let mut buffer = [0_i8; AV_ERROR_MAX_STRING_SIZE];
        let result = unsafe {
            ffmpeg::av_strerror(
                self,
                buffer.as_mut_ptr(),
                AV_ERROR_MAX_STRING_SIZE,
            )
        };

        if result < 0 {
            return "Unknown error".into();
        }

        let cstr = unsafe { CStr::from_ptr(buffer.as_ptr()) };
        String::from_utf8_lossy(cstr.to_bytes()).into_owned()
    }
}

impl ToResult for i32 {
    fn to_result(self) -> Result<Self, String> {
        if self < 0 {
            Err(self.to_error())
        } else {
            Ok(self)
        }
    }
}
