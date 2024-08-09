use super::{Classes, Icon, View};

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

pub struct Label<'a> {
    classes: Classes,
    icon: LabelIcon,
    text: Cow<'a, str>,
}

impl<'a> Label<'a> {
    pub fn new<T>(text: T, icon: LabelIcon) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        Self {
            classes: Classes::default(),
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

impl<'a> View for Label<'a> {
    fn classes_mut(&mut self) -> &mut Classes {
        &mut self.classes
    }
}

impl<'a> Render for Label<'a> {
    fn render(&self) -> Markup {
        html! {
            span class=[self.classes.get()] {
                @match self.icon {
                    LabelIcon::Icon(icon) => (icon.inline()),
                    LabelIcon::Object(id) => {
                        img src=(format!("/object/{id}/data")) .inline-icon;
                    }
                }

                span .label-text { (self.text) }
            }
        }
    }
}
