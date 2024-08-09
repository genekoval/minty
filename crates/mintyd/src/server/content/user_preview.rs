use super::{
    icon::{self, Icon},
    Label,
};

use maud::{html, Markup, Render};
use minty::Uuid;

const USER_ICON: Icon = icon::USER_CIRCLE;

fn deleted() -> Markup {
    html! {
        span {
            (USER_ICON)
            span .italic .label-text { "Deleted" }
        }
    }
}

#[derive(Debug)]
struct UserPreviewInner {
    id: Uuid,
    name: String,
}

impl UserPreviewInner {
    pub fn as_label(&self) -> impl Render + '_ {
        Label::icon(&self.name, USER_ICON)
    }

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
                (self.as_label())
            }
        }
    }
}

#[derive(Debug)]
pub struct UserPreview {
    inner: Option<UserPreviewInner>,
}

impl UserPreview {
    pub fn new(user: Option<minty::UserPreview>) -> Self {
        Self {
            inner: user.map(Into::into),
        }
    }

    pub fn as_label(&self) -> Markup {
        html! {
            @if let Some(user) = &self.inner {
                (user.as_label())
            } @else {
                (deleted())
            }
        }
    }
}

impl Render for UserPreview {
    fn render(&self) -> Markup {
        html! {
            @if let Some(user) = &self.inner {
                (user)
            } @else {
                (deleted())
            }
        }
    }
}
