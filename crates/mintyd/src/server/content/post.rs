use super::{icon, DateTime, IntoPage, PageTitle, UserPreview};

use maud::{html, Markup, Render};

#[derive(Debug)]
pub struct Post {
    title: String,
    description: String,
    poster: UserPreview,
    created: DateTime,
    modified: Option<DateTime>,
}

impl Post {
    fn title(&self) -> Markup {
        html! {
            @if !self.title.is_empty() {
                h1 { (self.title) }
            }
        }
    }

    fn description(&self) -> Markup {
        html! {
            @if !self.description.is_empty() {
                p { (self.description) }
            }
        }
    }

    fn poster(&self) -> Markup {
        html! {
            (self.poster)
        }
    }

    fn metadata(&self) -> Markup {
        html! {
            div .font-smaller ."leading-1-5" .secondary {
                div { (self.poster()) }

                div { (self.created) }

                @if let Some(modified) = self.modified {
                    div { (modified) }
                }
            }
        }
    }
}

impl From<minty::Post> for Post {
    fn from(value: minty::Post) -> Self {
        Self {
            title: value.title,
            description: value.description,
            poster: UserPreview::new(value.poster),
            created: DateTime::new(value.created)
                .icon(icon::CLOCK)
                .prefix("Posted"),
            modified: (value.modified != value.created).then(|| {
                DateTime::new(value.modified)
                    .icon(icon::PENCIL)
                    .prefix("Last modified")
            }),
        }
    }
}

impl Render for Post {
    fn render(&self) -> Markup {
        html! {
            (self.title())
            (self.metadata())
            (self.description())
        }
    }
}

impl PageTitle for Post {
    fn page_title(&self) -> &str {
        if self.title.is_empty() {
            "Untitled"
        } else {
            &self.title
        }
    }
}

impl IntoPage for minty::Post {
    type View = Post;
}
