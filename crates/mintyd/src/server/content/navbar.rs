use super::{icon, Html};

use maud::{html, Markup, Render};

#[derive(Debug)]
pub struct Navbar<'a, V> {
    page: &'a V,
}

impl<'a, V> Navbar<'a, V> {
    pub fn new(page: &'a V) -> Self {
        Self { page }
    }
}

impl<'a, V: Html> Render for Navbar<'a, V> {
    fn render(&self) -> Markup {
        html! {
            div {
                nav .flex-column {
                    a href="/" { (icon::HOME) }
                }

                main { (self.page.full()) }
            }
        }
    }
}
