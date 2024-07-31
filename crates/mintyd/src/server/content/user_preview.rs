use maud::{html, Markup, Render};
use minty::Uuid;

#[derive(Debug)]
pub struct UserPreview {
    id: Uuid,
    name: String,
}

impl UserPreview {
    fn path(&self) -> String {
        format!("/user/{}", self.id)
    }
}

impl From<minty::UserPreview> for UserPreview {
    fn from(value: minty::UserPreview) -> Self {
        Self {
            id: value.id,
            name: value.name,
        }
    }
}

impl Render for UserPreview {
    fn render(&self) -> Markup {
        html! {
            a href=(self.path()) { (self.name) }
        }
    }
}
