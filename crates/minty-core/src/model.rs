use minty::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize)]
pub struct About {
    pub version: &'static str,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Invitation {
    sub: Uuid,
}

impl Invitation {
    pub fn new(user: Uuid) -> Self {
        Self { sub: user }
    }

    pub fn user(&self) -> Uuid {
        self.sub
    }
}
