use super::{icon, DateTime, Html, Label, PostSearchResult, SourceList};

use maud::{html, Markup, Render};
use serde::{Serialize, Serializer};
use std::borrow::Cow;

#[derive(Debug)]
pub struct Tag {
    pub tag: minty::Tag,
    pub posts: Option<PostSearchResult>,
}

impl Tag {
    fn created(&self) -> impl Render {
        DateTime::new(self.tag.profile.created)
            .icon(icon::CALENDAR)
            .prefix("Created")
    }

    fn post_count(&self) -> String {
        format!(
            "{} Post{}",
            self.tag.post_count,
            match self.tag.post_count {
                1 => "",
                _ => "s",
            }
        )
    }
}

impl Serialize for Tag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.tag.serialize(serializer)
    }
}

impl Html for Tag {
    fn page_title(&self) -> Cow<str> {
        self.tag.profile.name.as_str().into()
    }

    fn full(&self) -> Markup {
        html! {
            h1 { (self.tag.profile.name) }

            @if !self.tag.profile.aliases.is_empty() {
                ul .bold {
                    @for alias in &self.tag.profile.aliases {
                        li { (alias) }
                    }
                }
            }

            @if !self.tag.profile.description.is_empty() {
                p { (self.tag.profile.description) }
            }

            (SourceList(&self.tag.profile.sources))

            .flex-column
            .gap-p5em
            .font-smaller
            .secondary
            .margin-top
            .margin-bottom
            {
                (self.created())
                span {
                    (Label::icon(self.post_count(), icon::FILE_IMAGE))
                }
            }

            @if let Some(posts) = &self.posts {
                (posts.full())
            }
        }
    }
}
