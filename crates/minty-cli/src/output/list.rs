use super::{color, HumanReadable};

use owo_colors::OwoColorize;
use serde::Serialize;
use std::io::{Result, Write};

const SEPARATOR: &str = " \u{00b7} ";
const SEPARATOR_LEN: usize = 3;

#[derive(Debug, Serialize)]
#[repr(transparent)]
pub struct List<'a, T>(&'a [T]);

pub trait SliceExt {
    type Output;

    fn list(&self) -> List<'_, Self::Output>;
}

impl<T> SliceExt for [T] {
    type Output = T;

    fn list(&self) -> List<'_, Self::Output> {
        List(self)
    }
}

impl<'a, T> HumanReadable for List<'a, T>
where
    T: HumanReadable,
{
    fn human_readable<W: Write>(&self, w: &mut W, indent: usize) -> Result<()> {
        let list = self.0;
        let digits = (list.len().ilog10() + 1) as usize;

        for (i, t) in self.0.iter().enumerate() {
            if i > 0 {
                writeln!(w)?;
            }

            let i = i + 1;

            write!(w, "{:>1$}", i.fg::<color::Index>(), digits)?;
            write!(w, "{}", SEPARATOR)?;

            t.human_readable(w, indent + digits + SEPARATOR_LEN)?;
        }

        Ok(())
    }
}
