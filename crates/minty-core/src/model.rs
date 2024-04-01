use crate::build;

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

impl Version {
    pub fn get() -> Self {
        Self {
            number: build::PKG_VERSION,
            branch: build::BRANCH,
            build_time: build::BUILD_TIME,
            build_os: build::BUILD_OS,
            build_type: build::BUILD_RUST_CHANNEL,
            commit_hash: build::COMMIT_HASH,
            commit_date: build::COMMIT_DATE,
            rust_version: build::RUST_VERSION,
            rust_channel: build::RUST_CHANNEL,
        }
    }
}
