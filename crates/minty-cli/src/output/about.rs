use super::HumanReadable;

use minty::{Url, Version};
use serde::Serialize;
use std::io::{Result, Write};

#[derive(Serialize)]
pub struct About<'a> {
    pub server: &'a str,
    pub url: &'a Url,
    #[serde(flatten)]
    pub info: minty::About,
}

impl<'a> HumanReadable for About<'a> {
    fn human_readable<W: Write>(
        &self,
        w: &mut W,
        _indent: usize,
    ) -> Result<()> {
        let alias = self.server;
        let url = self.url;
        let Version {
            number,
            branch,
            build_time,
            build_os,
            build_type,
            commit_hash,
            commit_date,
            rust_version,
            rust_channel,
        } = &self.info.version;

        writeln!(
            w,
            r#"{alias}: {url}
    Version: {number}
    Branch:  {branch}
    Build:
        Time: {build_time}
        OS:   {build_os}
        Type: {build_type}
    Commit:
        Hash: {commit_hash}
        Date: {commit_date}
    Rust:
        Version: {rust_version}
        Channel: {rust_channel}"#
        )
    }
}
