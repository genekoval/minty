use maud::{html, Markup, Render};

pub struct AvPlayerFooter;

impl Render for AvPlayerFooter {
    fn render(&self) -> Markup {
        html! {
            footer #av-player style="display: none;" _="on close \
                hide me then put '' into me then remove .with-player from body"
            {}
        }
    }
}
