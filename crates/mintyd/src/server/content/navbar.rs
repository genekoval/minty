use super::{icon, Html};

use maud::{html, Markup, Render};
use minty_core::{Cached, User};
use std::sync::Arc;

#[derive(Debug)]
pub struct Navbar<'a, V> {
    pub page: &'a V,
    pub user: Option<&'a Arc<Cached<User>>>,
}

impl<'a, V> Navbar<'a, V> {
    fn user_link(&self) -> Option<String> {
        self.user.map(|user| format!("/user/{}", user.id))
    }
}

impl<'a, V: Html> Render for Navbar<'a, V> {
    fn render(&self) -> Markup {
        html! {
            nav .flex-column {
                a href="/" { (icon::HOME) }

                @if let Some(link) = self.user_link() {
                    a href=(link) { (icon::CIRCLE_USER_ROUND) }
                } @else {
                    a href="/signin" { (icon::LOG_IN) }
                }
            }

            main { (self.page.full()) }
        }
    }
}
