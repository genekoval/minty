use maud::{html, Markup, Render};
use serde::Serialize;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Serialize)]
pub struct Post(pub minty::Post);

impl Post {
    fn title(&self) -> Markup {
        let title = self.0.title.as_str();

        html! {
            @if !title.is_empty() {
                h1 { (title) }
            }
        }
    }

    fn description(&self) -> Markup {
        let description = self.0.description.as_str();

        html! {
            @if !description.is_empty() {
                p { (description) }
            }
        }
    }
}

impl Display for Post {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let title = self.0.title.as_str();

        if title.is_empty() {
            f.write_str("Untitled")
        } else {
            f.write_str(title)
        }
    }
}

impl Render for Post {
    fn render(&self) -> Markup {
        html! {
            (self.title())
            (self.description())
        }
    }
}
