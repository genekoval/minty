use super::Icon;

use maud::{html, Markup, Render};
use std::borrow::Cow;

pub struct Label<'a> {
    icon: Icon,
    text: Cow<'a, str>,
}

impl<'a> Label<'a> {
    pub fn new<T>(text: T, icon: Icon) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        Self {
            icon,
            text: text.into(),
        }
    }
}

impl<'a> Render for Label<'a> {
    fn render(&self) -> Markup {
        html! {
            span {
                (self.icon.inline())
                span .label-text { (self.text) }
            }
        }
    }
}
