use super::{
    icon::{self, Icon},
    Label,
};

use maud::{html, Markup, Render};

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
struct UserPreviewInner<'a>(&'a minty::UserPreview);

impl<'a> UserPreviewInner<'a> {
    pub fn as_label(&self) -> impl Render + '_ {
        Label::icon(&self.0.name, USER_ICON)
    }

    fn path(&self) -> String {
        format!("/user/{}", self.0.id)
    }
}

impl<'a> Render for UserPreviewInner<'a> {
    fn render(&self) -> Markup {
        html! {
            a href=(self.path()) {
                (self.as_label())
            }
        }
    }
}

#[derive(Debug)]
pub struct UserPreview<'a>(Option<UserPreviewInner<'a>>);

impl<'a> UserPreview<'a> {
    pub fn new(user: Option<&'a minty::UserPreview>) -> Self {
        Self(user.map(UserPreviewInner))
    }

    pub fn as_label(&self) -> Markup {
        html! {
            @if let Some(user) = &self.0 {
                (user.as_label())
            } @else {
                (deleted())
            }
        }
    }
}

impl<'a> Render for UserPreview<'a> {
    fn render(&self) -> Markup {
        html! {
            @if let Some(user) = &self.0 {
                (user)
            } @else {
                (deleted())
            }
        }
    }
}
