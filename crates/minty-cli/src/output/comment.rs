use super::{
    color,
    icon::{self, Icon},
    metadata::Metadata,
    time::FormatDate,
    HumanReadable,
};

use minty::{Comment, CommentData};
use owo_colors::OwoColorize;
use std::{
    cmp,
    fmt::Display,
    io::{Result, Write},
};
use textwrap::termwidth;

const CORNER: char = '\u{250c}';
const HORIZONTAL: char = '\u{2500}';
const VERTICAL: char = '\u{2502}';

impl HumanReadable for Comment {
    fn human_readable<W: Write>(&self, w: &mut W, indent: usize) -> Result<()> {
        if let Some(user) = &self.user {
            write!(w, "{} ", icon::ACCOUNT)?;
            user.human_readable(w, indent + 2)?;
            writeln!(w)?;
        }

        Printer { out: w, level: 0 }.text(&self.content)?;
        writeln!(w)?;

        let parent = self
            .parent_id
            .map(|parent| format!("{parent} (Level {})", self.level));

        Metadata::new()
            .row("ID", icon::POUND, self.id)
            .row("Post", icon::DOCUMENT, self.post_id)
            .optional_row("Parent", icon::COMMENT, parent)
            .row("Created", icon::CLOCK, self.created.long_date())
            .print(indent, w)
    }
}

impl HumanReadable for CommentData {
    fn human_readable<W: Write>(
        &self,
        w: &mut W,
        _indent: usize,
    ) -> Result<()> {
        Printer {
            out: w,
            level: self.level,
        }
        .divider()?
        .metadata(|row| row.push(icon::POUND, self.id))?
        .metadata(|row| {
            row.push(
                icon::ACCOUNT,
                self.user
                    .as_ref()
                    .map(|user| user.name.as_str())
                    .unwrap_or("[deleted]"),
            )?
            .push(icon::CLOCK, self.created.relative_abbrev())
        })?
        .text(&self.content)?;

        Ok(())
    }
}

struct Printer<'w, W> {
    out: &'w mut W,
    level: u8,
}

impl<'w, W: Write> Printer<'w, W> {
    fn divider(&mut self) -> Result<&mut Self> {
        if self.level == 0 {
            return Ok(self);
        }

        self.indent_with_sep(self.level - 1)?;

        write!(self.out, "  {}", CORNER.fg::<color::Secodary>())?;

        for _ in 0..20 {
            write!(self.out, "{}", HORIZONTAL.fg::<color::Secodary>())?;
        }

        writeln!(self.out)?;
        Ok(self)
    }

    fn indent_with_sep(&mut self, level: u8) -> Result<&mut Self> {
        for _ in 0..level {
            write!(self.out, "  {}", VERTICAL.fg::<color::Secodary>())?;
        }

        Ok(self)
    }

    fn metadata<F>(&mut self, f: F) -> Result<&mut Self>
    where
        F: FnOnce(MetadataRow<'_, W>) -> Result<MetadataRow<'_, W>>,
    {
        self.row(|w| {
            f(MetadataRow::new(w))?;
            Ok(())
        })?;

        Ok(self)
    }

    fn row<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut W) -> Result<()>,
    {
        self.indent_with_sep(self.level)?;
        f(self.out)?;
        writeln!(self.out)
    }

    fn text(&mut self, text: &str) -> Result<&mut Self> {
        if text.is_empty() {
            self.row(|w| {
                write!(
                    w,
                    "{}",
                    format!("{} Deleted", icon::TRASH)
                        .fg::<color::Destructive>()
                )
            })?;
            return Ok(self);
        }

        let width = cmp::min(termwidth(), 80);
        textwrap::wrap(text, width)
            .into_iter()
            .try_for_each(|line| self.row(|w| write!(w, "{}", line)))?;

        Ok(self)
    }
}

struct MetadataRow<'w, W> {
    out: &'w mut W,
    first: bool,
}

impl<'w, W> MetadataRow<'w, W>
where
    W: Write,
{
    fn new(out: &'w mut W) -> Self {
        Self { out, first: true }
    }

    fn push<T: Display>(mut self, icon: Icon, value: T) -> Result<Self> {
        if self.first {
            self.first = false;
        } else {
            write!(self.out, "  ")?;
        }

        write!(
            self.out,
            "{} {}",
            icon.fg::<color::Label>(),
            value.fg::<color::Secodary>()
        )?;

        Ok(self)
    }
}
