use super::HumanReadable;

use minty::Url;
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
        let minty::About { version } = &self.info;

        writeln!(
            w,
            r#"{alias}: {url}
    server version: {version}"#
        )
    }
}
