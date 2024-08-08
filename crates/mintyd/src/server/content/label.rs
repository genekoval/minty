use super::{Classes, Icon, View};

use maud::{html, Markup, Render};
use std::borrow::Cow;

pub struct Label<'a> {
    classes: Classes,
    icon: Icon,
    text: Cow<'a, str>,
}

impl<'a> Label<'a> {
    pub fn new<T>(text: T, icon: Icon) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        Self {
            classes: Classes::default(),
            icon,
            text: text.into(),
        }
    }
}

impl<'a> View for Label<'a> {
    fn classes_mut(&mut self) -> &mut Classes {
        &mut self.classes
    }
}

impl<'a> Render for Label<'a> {
    fn render(&self) -> Markup {
        html! {
            span class=[self.classes.get()] {
                (self.icon.inline())
                span .label-text { (self.text) }
            }
        }
    }
}
