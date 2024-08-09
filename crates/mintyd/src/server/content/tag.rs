use super::{
    icon, DateTime, Html, IntoPage, Label, PageTitle, PostSearchResult,
    SourceList,
};

use maud::{html, Markup, Render};
use minty::Source;
use serde::{Serialize, Serializer};

#[derive(Debug)]
pub struct Tag {
    pub tag: minty::Tag,
    pub posts: Option<PostSearchResult>,
}

impl Serialize for Tag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.tag.serialize(serializer)
    }
}

impl IntoPage for Tag {
    type View = TagView;
}

#[derive(Debug)]
pub struct TagView {
    name: String,
    aliases: Vec<String>,
    description: String,
    sources: Vec<Source>,
    created: DateTime,
    post_count: u32,
    posts: Option<<PostSearchResult as IntoPage>::View>,
}

impl TagView {
    fn post_count(&self) -> String {
        format!(
            "{} Post{}",
            self.post_count,
            match self.post_count {
                1 => "",
                _ => "s",
            }
        )
    }
}

impl From<Tag> for TagView {
    fn from(value: Tag) -> Self {
        let tag = value.tag;
        let profile = tag.profile;

        Self {
            name: profile.name,
            aliases: profile.aliases,
            description: profile.description,
            sources: profile.sources,
            created: DateTime::new(profile.created)
                .icon(icon::CALENDAR)
                .prefix("Created"),
            post_count: tag.post_count,
            posts: value.posts.map(IntoPage::into_page),
        }
    }
}

impl PageTitle for TagView {
    fn page_title(&self) -> &str {
        &self.name
    }
}

impl Render for TagView {
    fn render(&self) -> Markup {
        html! {
            h1 { (self.name) }

            @if !self.aliases.is_empty() {
                ul .bold {
                    @for alias in &self.aliases {
                        li { (alias) }
                    }
                }
            }

            @if !self.description.is_empty() {
                p { (self.description) }
            }

            (SourceList(&self.sources))

            .flex-column
            .gap-p5em
            .font-smaller
            .secondary
            .margin-top
            .margin-bottom
            {
                (self.created)
                (Label::icon(self.post_count(), icon::FILE_IMAGE))
            }

            @if let Some(posts) = &self.posts {
                (posts.full())
            }
        }
    }
}
