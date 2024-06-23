use std::fmt::Debug;

use minty::{ErrorKind, Result};

pub trait ResultExt {
    type Value;

    fn expect_not_found(self);

    fn expect_unauthenticated(self);
}

impl<T: Debug> ResultExt for Result<T> {
    type Value = T;

    fn expect_not_found(self) {
        let err = self.expect_err("expected 'not found' error");
        let kind = err.kind();

        match kind {
            ErrorKind::NotFound => (),
            _ => {
                panic!("expected 'not found' error; got {kind:?} error: {err}")
            }
        }
    }

    fn expect_unauthenticated(self) {
        let err = self.expect_err("expected 'unauthenticated' error");
        let kind = err.kind();

        match kind {
            ErrorKind::Unauthenticated => (),
            _ => panic!(
                "expected 'unauthenticated' error; got {kind:?} error: {err}"
            ),
        }
    }
}
