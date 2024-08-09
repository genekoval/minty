use std::borrow::Cow;

type Class = Cow<'static, str>;

#[derive(Debug, Default)]
pub struct Classes(Vec<Class>);

impl Classes {
    pub fn get(&self) -> Option<String> {
        if self.0.is_empty() {
            None
        } else {
            Some(self.0.join(" "))
        }
    }

    fn push<T>(&mut self, class: T)
    where
        T: Into<Class>,
    {
        self.0.push(class.into());
    }
}

pub trait View: Sized {
    fn classes_mut(&mut self) -> &mut Classes;

    fn class<T>(mut self, class: T) -> Self
    where
        T: Into<Class>,
    {
        self.classes_mut().push(class);
        self
    }

    fn color(self, color: Color) -> Self {
        self.class(format!("fg-{}", color.0))
    }
}

pub struct Color(&'static str);

pub mod color {
    use super::Color;

    pub const PURPLE: Color = Color("purple");
}
