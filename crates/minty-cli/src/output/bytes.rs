use super::num::FormatNumber;

use bytesize::{self, KIB};

pub trait ByteSize {
    fn to_bytestring(self) -> String;
}

impl ByteSize for u64 {
    fn to_bytestring(self) -> String {
        if self < KIB {
            format!("{} B", self)
        } else {
            format!(
                "{} ({} bytes)",
                bytesize::to_string(self, true),
                self.format()
            )
        }
    }
}
