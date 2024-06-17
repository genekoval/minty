use super::{
    bytes::ByteSize, color, icon, metadata::Metadata, time::FormatDate,
    HumanReadable,
};

use minty::{Object, ObjectError, ObjectPreview};
use owo_colors::OwoColorize;
use std::io::{Result, Write};

impl HumanReadable for Object {
    fn human_readable<W: Write>(&self, w: &mut W, indent: usize) -> Result<()> {
        writeln!(w, "Posts {}", self.posts.len().fg::<color::Result>())?;

        for (i, post) in self.posts.iter().enumerate() {
            write!(w, "  {} ", (i + 1).fg::<color::Index>())?;
            post.human_readable(w, 4)?;
            writeln!(w)?;
        }

        Metadata::new()
            .row("ID", icon::POUND, self.id)
            .optional_row("Preview", icon::IMAGE, self.preview_id)
            .row("SHA256", icon::BINARY, self.hash.as_str())
            .row("Size", icon::HARDDISK, self.size.to_bytestring())
            .row(
                "Type",
                icon::IMAGE,
                format!("{}/{}", self.r#type, self.subtype),
            )
            .row("Added", icon::CLOCK, self.added.long_date())
            .print(indent, w)
    }
}

impl HumanReadable for ObjectError {
    fn human_readable<W: Write>(&self, w: &mut W, indent: usize) -> Result<()> {
        write!(w, "{:1$}", "", indent)?;
        writeln!(w, "{}: {}", self.id, self.message)
    }
}

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
