use super::{color, HumanReadable};

use minty::SearchResult;
use owo_colors::OwoColorize;
use std::io::{Result, Write};

const SEPARATOR: &str = " \u{00b7} ";
const SEPARATOR_LEN: usize = 3;

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
            let digits = (self.hits.len().ilog10() + 1) as usize;

            for (i, hit) in self.hits.iter().enumerate() {
                let i = i + 1;

                write!(w, "{:>1$}", i.fg::<color::Index>(), digits)?;
                write!(w, "{}", SEPARATOR)?;

                hit.human_readable(w, indent + digits + SEPARATOR_LEN)?;
                writeln!(w)?;
            }
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
