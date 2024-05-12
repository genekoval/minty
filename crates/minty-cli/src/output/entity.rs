use super::{
    text::StringExt,
    view::{heading, links, list},
    HumanReadable,
};

use minty::{EntityProfile, ProfileName};
use owo_colors::Style;
use std::io::{Result, Write};

impl HumanReadable for EntityProfile {
    fn human_readable<W: Write>(&self, w: &mut W, indent: usize) -> Result<()> {
        heading(w, &self.name)?;
        writeln!(w)?;

        list(w, &self.aliases, Some(Style::new().italic()))?;

        self.description.wrapped().human_readable(w, indent)?;

        links(w, &self.sources)?;

        Ok(())
    }
}

impl HumanReadable for ProfileName {
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
