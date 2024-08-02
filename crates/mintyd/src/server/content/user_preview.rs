use super::icon::{self, Icon};

use maud::{html, Markup, Render};
use minty::Uuid;

const USER_ICON: Icon = icon::USER_CIRCLE;

#[derive(Debug)]
struct UserPreviewInner {
    id: Uuid,
    name: String,
}

impl UserPreviewInner {
    fn path(&self) -> String {
        format!("/user/{}", self.id)
    }
}

impl From<minty::UserPreview> for UserPreviewInner {
    fn from(value: minty::UserPreview) -> Self {
        Self {
            id: value.id,
            name: value.name,
        }
    }
}

impl Render for UserPreviewInner {
    fn render(&self) -> Markup {
        html! {
            a href=(self.path()) {
                (USER_ICON)
                span ."label-text" { (self.name) }
            }
        }
    }
}

#[derive(Debug)]
pub struct UserPreview {
    inner: Option<UserPreviewInner>,
    font_smaller: bool,
    secondary: bool,
}

impl UserPreview {
    pub fn new(user: Option<minty::UserPreview>) -> Self {
        Self {
            inner: user.map(Into::into),
            font_smaller: false,
            secondary: false,
        }
    }

    pub fn font_smaller(mut self) -> Self {
        self.font_smaller = true;
        self
    }

    pub fn secondary(mut self) -> Self {
        self.secondary = true;
        self
    }
}

impl Render for UserPreview {
    fn render(&self) -> Markup {
        html! {
            span .font-smaller[self.font_smaller] .secondary[self.secondary] {
                @if let Some(user) = &self.inner {
                    (user)
                } @else {
                    span {
                        (USER_ICON)
                        span .italic ."label-text" { "Deleted" }
                    }
                }
            }
        }
    }
}
