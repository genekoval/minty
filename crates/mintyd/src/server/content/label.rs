use super::Icon;

use maud::{html, Markup, Render};
use minty::Uuid;
use std::borrow::Cow;

#[derive(Clone, Copy, Debug)]
pub enum LabelIcon {
    Icon(Icon),
    Object(Uuid),
}

impl From<Icon> for LabelIcon {
    fn from(value: Icon) -> Self {
        Self::Icon(value)
    }
}

impl From<Uuid> for LabelIcon {
    fn from(value: Uuid) -> Self {
        Self::Object(value)
    }
}

impl Render for LabelIcon {
    fn render(&self) -> Markup {
        html! {
            @match self {
                Self::Icon(icon) => (icon.inline()),
                Self::Object(id) => {
                    img src=(format!("/object/{id}/data")) .inline-icon;
                }
            }
        }
    }
}

pub struct Label<'a> {
    icon: LabelIcon,
    text: Cow<'a, str>,
}

impl<'a> Label<'a> {
    pub fn new<T>(text: T, icon: LabelIcon) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        Self {
            icon,
            text: text.into(),
        }
    }

    pub fn icon<T>(text: T, icon: Icon) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        Self::new(text, LabelIcon::Icon(icon))
    }
}

impl<'a> Render for Label<'a> {
    fn render(&self) -> Markup {
        html! {
            (self.icon)
            span .label-text { (self.text) }
        }
    }
}
