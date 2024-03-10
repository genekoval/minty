use super::{
    color, icon,
    metadata::Metadata,
    time::FormatDate,
    view::{heading, links, list},
    HumanReadable,
};

use minty::{Tag, TagName, TagPreview};
use owo_colors::{OwoColorize, Style};
use std::io::{Result, Write};

impl HumanReadable for Tag {
    fn human_readable<W: Write>(&self, w: &mut W, indent: usize) -> Result<()> {
        heading(w, &self.name)?;
        list(w, &self.aliases, Some(Style::new().italic()))?;

        if !self.description.is_empty() {
            writeln!(w, "\n{}", self.description)?;
        }

        if !self.sources.is_empty() {
            writeln!(w)?;
            links(w, &self.sources)?;
        }

        writeln!(w)?;

        Metadata::new()
            .row("ID", icon::POUND, self.id)
            .row("Posts", icon::DOCUMENT, self.post_count)
            .row("Created", icon::CALENDAR, self.created.long_date())
            .print(indent, w)
    }
}

impl HumanReadable for TagName {
    fn human_readable<W: Write>(
        &self,
        w: &mut W,
        _indent: usize,
    ) -> Result<()> {
        writeln!(w, "{}", self.name)?;

        for alias in &self.aliases {
            writeln!(w, "    {alias}")?;
        }

        Ok(())
    }
}

impl HumanReadable for TagPreview {
    fn human_readable<W: Write>(&self, w: &mut W, indent: usize) -> Result<()> {
        writeln!(w, "{}", self.name.bold())?;

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
