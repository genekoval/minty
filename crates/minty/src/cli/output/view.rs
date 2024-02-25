use minty::Source;
use owo_colors::{colors::css::*, OwoColorize, Style};
use std::{
    fmt::Display,
    io::{Result, Write},
};

pub fn heading<W: Write>(write: &mut W, text: &str) -> Result<()> {
    writeln!(write, "{}", text.bold())
}

pub fn links<W: Write>(write: &mut W, links: &[Source]) -> Result<()> {
    let style = Style::new().fg::<SteelBlue>();
    list(write, links, Some(style))
}

pub fn list<W, T>(
    write: &mut W,
    items: &[T],
    style: Option<Style>,
) -> Result<()>
where
    W: Write,
    T: Display,
{
    if !items.is_empty() {
        for item in items {
            let item = item.style(style.unwrap_or_default());
            writeln!(write, "\u{eab6} {item}")?;
        }
    }

    Ok(())
}

pub fn metadata_row<W, T>(
    write: &mut W,
    key: &str,
    value: T,
    alignment: usize,
) -> Result<()>
where
    W: Write,
    T: Display,
{
    write!(write, "{}", key.fg::<Violet>())?;

    let len = key.chars().count();
    let spacing = alignment - len + 2;

    for _ in 0..spacing {
        write.write_all(" ".as_bytes())?;
    }

    writeln!(write, "{value}")
}

#[macro_export]
macro_rules! metadata {
    ($write:expr, $(($key:literal, $value:expr)),*) => {
        {
            let mut len = 0;

            $({
                let key_len = $key.chars().count();
                if key_len > len {
                    len = key_len;
                }
            })*

            $($crate::cli::output::view::metadata_row(
                $write,
                $key,
                $value,
                len
            )?;)*
        }
    };
}

pub use metadata;
