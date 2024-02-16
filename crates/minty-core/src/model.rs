use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct About {
    pub version: Version,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Version {
    pub number: &'static str,
    pub branch: &'static str,
    pub build_time: &'static str,
    pub build_os: &'static str,
    pub build_type: &'static str,
    pub commit_hash: &'static str,
    pub commit_date: &'static str,
    pub rust_version: &'static str,
    pub rust_channel: &'static str,
}
