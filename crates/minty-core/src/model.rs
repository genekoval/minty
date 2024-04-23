use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct About {
    pub version: &'static str,
}
