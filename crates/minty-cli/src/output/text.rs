use super::HumanReadable;

use std::{
    cmp::min,
    io::{Result, Write},
};
use textwrap::{fill, termwidth, Options};

pub struct WrappedText<'a>(&'a str);

impl<'a> HumanReadable for WrappedText<'a> {
    fn human_readable<W: Write>(
        &self,
        w: &mut W,
        _indent: usize,
    ) -> Result<()> {
        let text = self.0;

        if text.is_empty() {
            return Ok(());
        }

        let width = min(termwidth(), 80);
        let options = Options::new(width);

        let text = fill(text, options);

        writeln!(w, "{}\n", text)
    }
}

pub trait StringExt {
    fn wrapped(&self) -> WrappedText<'_>;
}

impl StringExt for String {
    fn wrapped(&self) -> WrappedText<'_> {
        WrappedText(self)
    }
}
