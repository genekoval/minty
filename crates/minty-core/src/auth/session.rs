use base64::{engine::general_purpose::STANDARD_NO_PAD as Base64, Engine};
use rand::{rngs::OsRng, RngCore};
use std::{
    fmt::{self, Display},
    result,
    str::{self, FromStr},
};

pub use base64::DecodeSliceError as Base64DecodeError;

/// The number of bytes in a session ID.
const SESSION_ID_LENGTH: usize = 32;

// Every 3 bytes of binary data encodes to 4 base64 characters
const SESSION_ID_STR_LEN: usize = SESSION_ID_LENGTH.div_ceil(3) * 4;

#[derive(Clone, Copy, Debug)]
pub struct SessionId([u8; SESSION_ID_LENGTH]);

impl SessionId {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        let mut buf = [0u8; SESSION_ID_LENGTH];
        OsRng.fill_bytes(&mut buf);
        Self(buf)
    }
}

impl Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = [0u8; SESSION_ID_STR_LEN];

        let bytes_written = Base64
            .encode_slice(self.0, &mut buf)
            .expect("buffer should be large enough");

        let encoded =
            unsafe { str::from_utf8_unchecked(&buf[..bytes_written]) };

        f.write_str(encoded)
    }
}

impl FromStr for SessionId {
    type Err = Base64DecodeError;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        let mut buf = [0u8; SESSION_ID_LENGTH];
        Base64.decode_slice(s, &mut buf)?;

        Ok(Self(buf))
    }
}
