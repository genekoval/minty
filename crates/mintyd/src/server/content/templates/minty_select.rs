use super::style;

use maud::{html, Markup, Render};

pub struct MintySelect;

impl Render for MintySelect {
    fn render(&self) -> Markup {
        html! {
            template #minty-select-template {
                (style!("minty_select.css"))

                button .closed {
                    slot {}
                }

                .menu {}
            }
        }
    }
}
