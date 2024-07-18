use super::{color, HumanReadable, SliceExt};

use minty::SearchResult;
use owo_colors::OwoColorize;
use std::io::{Result, Write};

impl<T> HumanReadable for SearchResult<T>
where
    T: HumanReadable,
{
    fn human_readable<W: Write>(&self, w: &mut W, indent: usize) -> Result<()> {
        if self.total == 0 {
            writeln!(w, "No matches found")?;
            return Ok(());
        }

        if !self.hits.is_empty() {
            self.hits.list().human_readable(w, indent)?;
            writeln!(w)?;
        }

        writeln!(
            w,
            "{} of {} results",
            self.hits.len().fg::<color::Result>(),
            self.total.fg::<color::Result>(),
        )?;

        Ok(())
    }
}
