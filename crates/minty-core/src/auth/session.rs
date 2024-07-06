pub use base64::DecodeSliceError as Base64DecodeError;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD as Base64, Engine};
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest as _, Sha256};
use std::{
    fmt::{self, Display},
    ops::Deref,
    result,
    str::{self, FromStr},
};

macro_rules! byte_array {
    ($t:ident, $n:expr) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        #[repr(transparent)]
        pub struct $t([u8; $n]);

        impl Deref for $t {
            type Target = [u8];

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl Display for $t {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                const ENCODED_LEN: usize = match base64::encoded_len($n, false)
                {
                    Some(len) => len,
                    None => panic!("type too large to base64 encode"),
                };

                let mut buf = [0u8; ENCODED_LEN];

                let bytes_written = Base64
                    .encode_slice(self.0, &mut buf)
                    .expect("buffer should be large enough");

                let encoded =
                    unsafe { str::from_utf8_unchecked(&buf[..bytes_written]) };

                f.write_str(encoded)
            }
        }

        impl FromStr for $t {
            type Err = Base64DecodeError;

            fn from_str(s: &str) -> result::Result<Self, Self::Err> {
                let mut buf = [0u8; $n];
                Base64.decode_slice(s, &mut buf)?;

                Ok(Self(buf))
            }
        }
    };
}

macro_rules! random_byte_array {
    ($t:ident, $n:expr) => {
        byte_array!($t, $n);

        impl $t {
            pub fn generate() -> Self {
                let mut buf = [0u8; $n];
                OsRng.fill_bytes(&mut buf);
                Self(buf)
            }

            pub fn digest(&self) -> Digest {
                Digest(Sha256::digest(self.0).into())
            }
        }
    };
}

byte_array!(Digest, 32);

random_byte_array!(SessionId, 32);
