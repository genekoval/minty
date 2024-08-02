use maud::{html, Markup, PreEscaped, Render};

pub struct Space;

impl Render for Space {
    fn render(&self) -> Markup {
        html! {
            (PreEscaped("&nbsp;"))
        }
    }
}
