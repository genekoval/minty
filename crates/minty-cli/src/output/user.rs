use super::{color, icon, metadata::Metadata, time::FormatDate, HumanReadable};

use minty::{User, UserPreview};
use owo_colors::OwoColorize;
use std::io::{Result, Write};

impl HumanReadable for User {
    fn human_readable<W: Write>(&self, w: &mut W, indent: usize) -> Result<()> {
        self.profile.human_readable(w, indent)?;

        Metadata::new()
            .row("ID", icon::POUND, self.id)
            .row("Email", icon::EMAIL, &self.email)
            .row("Posts", icon::DOCUMENT, self.post_count)
            .row("Comments", icon::COMMENT, self.comment_count)
            .row("Tags", icon::TAG, self.tag_count)
            .row("Joined", icon::CALENDAR, self.profile.created.long_date())
            .print(indent, w)
    }
}

impl HumanReadable for UserPreview {
    fn human_readable<W: Write>(&self, w: &mut W, indent: usize) -> Result<()> {
        writeln!(w, "{}", self.name.bold())?;

        write!(w, "{:1$}", "", indent)?;
        writeln!(
            w,
            "{} {}",
            icon::POUND.fg::<color::Label>(),
            self.id.fg::<color::Secodary>()
        )
    }
}
