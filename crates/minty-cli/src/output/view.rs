use super::color;

use minty::Source;
use owo_colors::{OwoColorize, Style};
use std::{
    fmt::Display,
    io::{Result, Write},
};

pub fn heading<W: Write>(write: &mut W, text: &str) -> Result<()> {
    writeln!(write, "{}", text.bold())
}

pub fn links<W: Write>(write: &mut W, links: &[Source]) -> Result<()> {
    let style = Style::new().fg::<color::Link>();
    list(write, links, Some(style))
}

pub fn list<W, T>(w: &mut W, items: &[T], style: Option<Style>) -> Result<()>
where
    W: Write,
    T: Display,
{
    if !items.is_empty() {
        for item in items {
            let item = item.style(style.unwrap_or_default());
            writeln!(w, "\u{eab6} {item}")?;
        }
    }

    Ok(())
}
