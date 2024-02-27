use super::{
    time::FormatDate,
    view::{heading, links, list, metadata},
    HumanReadable,
};

use minty::Tag;
use owo_colors::Style;
use std::io::{Result, Write};

impl HumanReadable for Tag {
    fn human_readable<W: Write>(&self, mut write: W) -> Result<()> {
        heading(&mut write, &self.name)?;
        list(&mut write, &self.aliases, Some(Style::new().italic()))?;

        if !self.description.is_empty() {
            writeln!(&mut write, "\n{}", self.description)?;
        }

        if !self.sources.is_empty() {
            writeln!(&mut write)?;
            links(&mut write, &self.sources)?;
        }

        writeln!(&mut write)?;
        metadata!(
            &mut write,
            ("\u{f0423} ID", &self.id),
            ("\u{f0219} Posts", &self.post_count),
            ("\u{eab0} Created", &self.created.long_date())
        );

        Ok(())
    }
}
