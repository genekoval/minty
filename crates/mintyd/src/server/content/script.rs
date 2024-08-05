use maud::{html, Markup, Render};

/// Links to an external script at the given path.
pub struct Script(pub &'static str);

impl Render for Script {
    fn render(&self) -> Markup {
        html! {
            script src=(self.0) {}
        }
    }
}
