use maud::{html, Markup, Render};
use serde::Serialize;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Serialize)]
pub struct Post(pub minty::Post);

impl Post {
    fn title(&self) -> &str {
        let title = self.0.title.as_str();

        if title.is_empty() {
            "Untitled"
        } else {
            title
        }
    }
}

impl Display for Post {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.title())
    }
}

impl Render for Post {
    fn render(&self) -> Markup {
        html! {
            h1 { (self.title()) }
        }
    }
}
