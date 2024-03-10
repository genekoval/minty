use std::fmt::{self, Display, Write};

#[derive(Clone, Copy, Debug)]
pub struct Icon(char);

impl Display for Icon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char(self.0)
    }
}

/// nf-md-calendar 󰃭
pub const CALENDAR: Icon = Icon('\u{f00ed}');

/// nf-md-clock 󰥔
pub const CLOCK: Icon = Icon('\u{f0954}');

/// nf-md-comment 󰅺
pub const COMMENT: Icon = Icon('\u{f017a}');

/// nf-md-file_document 󰈙
pub const DOCUMENT: Icon = Icon('\u{f0219}');

/// nf-md-eye 󰈈
pub const EYE: Icon = Icon('\u{f0208}');

/// nf-md-file_image 󰈟
pub const IMAGE: Icon = Icon('\u{f021f}');

/// nf-md-pencil 󰏫
pub const PENCIL: Icon = Icon('\u{f03eb}');

/// nf-md-pound 󰐣
pub const POUND: Icon = Icon('\u{f0423}');
