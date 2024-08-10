use super::{
    icon, DateTime, Html, Label, ObjectGrid, PostPreview, UserPreview,
};

use maud::{html, Markup, Render};
use serde::{Serialize, Serializer};

#[derive(Debug)]
pub struct Post(pub minty::Post);

impl Post {
    fn title(&self) -> Markup {
        html! {
            @if !self.0.title.is_empty() {
                h1 { (self.0.title) }
            }
        }
    }

    fn description(&self) -> Markup {
        html! {
            @if !self.0.description.is_empty() {
                p { (self.0.description) }
            }
        }
    }

    fn poster(&self) -> impl Render + '_ {
        UserPreview::new(self.0.poster.as_ref())
    }

    fn created(&self) -> impl Render {
        DateTime::new(self.0.created)
            .icon(icon::CLOCK)
            .prefix("Posted")
    }

    fn modified(&self) -> Option<impl Render> {
        (self.0.modified != self.0.created).then(|| {
            DateTime::new(self.0.modified)
                .icon(icon::PENCIL)
                .prefix("Last modified")
        })
    }

    fn metadata(&self) -> Markup {
        html! {
            div .font-smaller ."leading-1-5" .secondary {
                div { (self.poster()) }

                div { (self.created()) }

                @if let Some(modified) = self.modified() {
                    div { (modified) }
                }
            }
        }
    }

    fn objects(&self) -> impl Render + '_ {
        ObjectGrid(&self.0.objects)
    }

    fn posts(&self) -> Markup {
        html! {
            @if !self.0.posts.is_empty() {
                .flex-column .margin-top {
                    @for post in &self.0.posts {
                        (PostPreview(post))
                    }
                }
            }
        }
    }

    fn tags(&self) -> Markup {
        html! {
            @if !self.0.tags.is_empty() {
                .tags .flex-row .flex-wrap {
                    @for tag in &self.0.tags {
                        a href=(format!("/tag/{}", tag.id)) {
                            (Label::icon(&tag.name, icon::HASH))
                        }
                    }
                }
            }
        }
    }
}

impl From<minty::Post> for Post {
    fn from(value: minty::Post) -> Self {
        Self(value)
    }
}

impl Serialize for Post {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl Html for Post {
    fn page_title(&self) -> &str {
        let title = self.0.title.as_str();

        if title.is_empty() {
            "Untitled"
        } else {
            title
        }
    }

    fn full(&self) -> Markup {
        html! {
            (self.title())
            (self.metadata())
            (self.description())
            (self.objects())
            (self.posts())
            (self.tags())
        }
    }
}
