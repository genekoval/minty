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

pub fn metadata_row<W, T>(
    w: &mut W,
    title: &str,
    icon: &str,
    value: T,
    alignment: usize,
) -> Result<()>
where
    W: Write,
    T: Display,
{
    write!(
        w,
        "{} {}",
        icon.fg::<color::Label>(),
        title.fg::<color::Label>()
    )?;

    let len = title.chars().count();
    let spacing = alignment - len + 2;

    for _ in 0..spacing {
        w.write_all(" ".as_bytes())?;
    }

    writeln!(w, "{value}")
}

#[macro_export]
macro_rules! metadata {
    ($write:expr, $(($title:literal, $icon:expr, $value:expr)),*) => {
        {
            let mut len = 0;

            $({
                let key_len = $title.chars().count();
                if key_len > len {
                    len = key_len;
                }
            })*

            $($crate::output::view::metadata_row(
                $write,
                $title,
                $icon,
                $value,
                len
            )?;)*
        }
    };
}

pub use metadata;
