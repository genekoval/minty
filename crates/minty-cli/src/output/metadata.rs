use super::{color, icon::Icon};

use owo_colors::OwoColorize;
use std::{
    cmp,
    fmt::Display,
    io::{Result, Write},
};

#[derive(Debug, Default)]
pub struct Metadata {
    rows: Vec<Row>,
    alignment: usize,
}

impl Metadata {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn row<T: Display>(
        mut self,
        title: &'static str,
        icon: Icon,
        value: T,
    ) -> Self {
        self.alignment = cmp::max(self.alignment, title.as_bytes().len());

        self.rows.push(Row {
            title,
            icon,
            value: value.to_string(),
        });

        self
    }

    pub fn optional_row<T: Display>(
        self,
        title: &'static str,
        icon: Icon,
        value: Option<T>,
    ) -> Self {
        if let Some(value) = value {
            self.row(title, icon, value)
        } else {
            self
        }
    }

    pub fn print<W: Write>(self, indent: usize, w: &mut W) -> Result<()> {
        self.rows
            .into_iter()
            .try_for_each(|row| row.print(indent, self.alignment, w))
    }
}

#[derive(Debug)]
struct Row {
    title: &'static str,
    icon: Icon,
    value: String,
}

impl Row {
    fn print<W: Write>(
        self,
        indent: usize,
        aligment: usize,
        w: &mut W,
    ) -> Result<()> {
        let icon = self.icon.fg::<color::Label>();
        let title = self.title.fg::<color::Label>();
        let value = self.value.as_str();

        writeln!(w, "{:1$}{icon} {title:2$}  {value}", "", indent, aligment)
    }
}
