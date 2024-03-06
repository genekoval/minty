use super::{
    badge, color,
    time::FormatDate,
    view::{heading, links, list, metadata},
    HumanReadable,
};

use minty::{Tag, TagName, TagPreview};
use owo_colors::{OwoColorize, Style};
use std::io::{Result, Write};

impl HumanReadable for Tag {
    fn human_readable<W: Write>(
        &self,
        w: &mut W,
        _indent: usize,
    ) -> Result<()> {
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
        metadata!(
            w,
            ("ID", badge::POUND, &self.id),
            ("Posts", badge::DOCUMENT, &self.post_count),
            ("Created", badge::CALENDAR, &self.created.long_date())
        );

        Ok(())
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
            badge::POUND.fg::<color::Label>(),
            self.id.fg::<color::Secodary>()
        )?;

        Ok(())
    }
}
