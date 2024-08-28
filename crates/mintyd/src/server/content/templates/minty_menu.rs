use super::style;

use maud::{html, Markup, Render};

pub struct MintyMenu;

impl Render for MintyMenu {
    fn render(&self) -> Markup {
        html! {
            template #minty-menu-template {
                (style!("minty_menu.css"))

                button .closed {
                    slot name="menu-button" { "Menu" }
                }

                .menu {
                    slot {}
                }
            }
        }
    }
}
