use super::{IntoPage, PageTitle, UserPreview};

use maud::{html, Markup, Render};

#[derive(Debug)]
pub struct Post {
    title: String,
    description: String,
    poster: Option<UserPreview>,
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
            @if let Some(poster) = &self.poster {
                (poster)
            } @else {
                span { "Deleted" }
            }
        }
    }
}

impl From<minty::Post> for Post {
    fn from(value: minty::Post) -> Self {
        Self {
            title: value.title,
            description: value.description,
            poster: value.poster.map(Into::into),
        }
    }
}

impl Render for Post {
    fn render(&self) -> Markup {
        html! {
            div { (self.poster()) }
            (self.title())
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
