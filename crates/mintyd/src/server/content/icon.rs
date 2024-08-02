use maud::{html, Markup, PreEscaped, Render};

#[derive(Clone, Copy, Debug)]
pub struct Icon(&'static str);

impl Icon {
    fn svg(&self) -> &str {
        // Remove any trailing newlines from SVG file contents
        self.0.trim_end()
    }
}

impl Render for Icon {
    fn render(&self) -> Markup {
        html! {
            (PreEscaped(self.svg()))
        }
    }
}

macro_rules! icon {
    ($name:literal) => {
        Icon(include_str!(concat!("icon/", $name, ".svg")))
    };
}

pub const CLOCK: Icon = icon!("clock");
pub const PENCIL: Icon = icon!("pencil");
pub const USER_CIRCLE: Icon = icon!("user_circle");
