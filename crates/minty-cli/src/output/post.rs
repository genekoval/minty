use super::{
    color, icon,
    metadata::Metadata,
    text::StringExt,
    time::{FormatDate, RelativeUnits},
    HumanReadable,
};

use minty::{Post, PostPreview};
use owo_colors::OwoColorize;
use std::io::{Result, Write};

impl HumanReadable for Post {
    fn human_readable<W: Write>(&self, w: &mut W, indent: usize) -> Result<()> {
        if let Some(poster) = &self.poster {
            write!(w, "{} ", icon::ACCOUNT)?;
            poster.human_readable(w, indent + 2)?;
            writeln!(w)?;
        }

        if !self.title.is_empty() {
            writeln!(w, "{}\n", self.title.italic().fg::<color::Title>())?;
        }

        self.description.wrapped().human_readable(w, indent)?;

        if !self.objects.is_empty() {
            writeln!(
                w,
                "Objects {}",
                self.objects.len().fg::<color::Result>()
            )?;

            for (i, object) in self.objects.iter().enumerate() {
                write!(w, "  {} ", i + 1)?;
                object.human_readable(w, 4)?;
                writeln!(w)?;
            }
        }

        if !self.posts.is_empty() {
            writeln!(w, "Posts {}", self.posts.len().fg::<color::Result>())?;

            for post in &self.posts {
                write!(w, "  \u{eab6} ")?;
                post.human_readable(w, 4)?;
                writeln!(w)?;
            }
        }

        if !self.tags.is_empty() {
            writeln!(w, "Tags {}", self.tags.len().fg::<color::Result>())?;

            for tag in &self.tags {
                write!(w, "  \u{eab6} ")?;
                tag.human_readable(w, 4)?;
                writeln!(w)?;
            }
        }

        let mut metadata = Metadata::new()
            .row("ID", icon::POUND, self.id)
            .row("Visibility", icon::EYE, self.visibility)
            .row("Comments", icon::COMMENT, self.comment_count)
            .row("Created", icon::CLOCK, self.created.long_date());

        if self.modified != self.created {
            metadata = metadata.row(
                "Modified",
                icon::PENCIL,
                self.modified.long_date(),
            );
        }

        metadata.print(indent, w)
    }
}

impl HumanReadable for PostPreview {
    fn human_readable<W: Write>(&self, w: &mut W, indent: usize) -> Result<()> {
        if self.title.is_empty() {
            writeln!(w, "{}", "Untitled".italic())?;
        } else {
            writeln!(w, "{}", self.title.bold())?;
        }

        write!(w, "{:1$}", "", indent)?;
        writeln!(
            w,
            "{} {}",
            icon::POUND.fg::<color::Label>(),
            self.id.fg::<color::Secodary>()
        )?;

        write!(w, "{:1$}", "", indent)?;
        write!(
            w,
            "{} {}",
            icon::IMAGE.fg::<color::Label>(),
            self.object_count.fg::<color::Secodary>()
        )?;
        write!(
            w,
            "  {} {}",
            icon::COMMENT.fg::<color::Label>(),
            self.comment_count.fg::<color::Secodary>()
        )?;
        write!(
            w,
            "  {} {}",
            icon::CLOCK.fg::<color::Label>(),
            self.created.relative_abbrev(1).fg::<color::Secodary>()
        )?;
        writeln!(w)?;

        Ok(())
    }
}
