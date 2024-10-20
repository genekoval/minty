use super::{Classes, View};

use maud::{html, Markup, Render};
use std::borrow::Cow;

pub struct NavigationLink<'a, V> {
    classes: Classes,
    path: Cow<'a, str>,
    content: V,
}

impl<'a, V> NavigationLink<'a, V> {
    pub fn new<T>(path: T, content: V) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        Self {
            classes: Classes::default(),
            path: path.into(),
            content,
        }
    }
}

impl<'a, V> Render for NavigationLink<'a, V>
where
    V: Render,
{
    fn render(&self) -> Markup {
        html! {
            a href=(self.path)
                hx-get=(self.path)
                hx-target="main"
                hx-push-url="true"
                class=[self.classes.get()]
            {
                (self.content)
            }
        }
    }
}

impl<'a, V> View for NavigationLink<'a, V> {
    fn classes_mut(&mut self) -> &mut Classes {
        &mut self.classes
    }
}
