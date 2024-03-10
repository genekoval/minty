use super::{color, icon, HumanReadable};

use minty::ObjectPreview;
use owo_colors::OwoColorize;
use std::io::{Result, Write};

impl HumanReadable for ObjectPreview {
    fn human_readable<W: Write>(&self, w: &mut W, indent: usize) -> Result<()> {
        let media_type = format!("{}/{}", self.r#type, self.subtype);

        writeln!(w, "{media_type}")?;

        write!(w, "{:1$}", "", indent)?;
        writeln!(
            w,
            "{} {}",
            icon::POUND.fg::<color::Label>(),
            self.id.fg::<color::Secodary>()
        )?;

        Ok(())
    }
}
