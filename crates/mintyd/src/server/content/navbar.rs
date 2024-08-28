use super::{icon, Html};

use maud::{html, Markup, Render};
use minty::UserPreview;

#[derive(Debug)]
pub struct Navbar<'a, V> {
    pub page: &'a V,
    pub user: Option<UserPreview>,
}

impl<'a, V: Html> Render for Navbar<'a, V> {
    fn render(&self) -> Markup {
        html! {
            nav .flex-column {
                .nav-primary .nav-section {
                    a href="/" { (icon::HOME) }

                }

                .nav-secondary .nav-section {
                    @if let Some(user) = &self.user {
                        minty-menu {
                            span slot="menu-button" {
                                (icon::CIRCLE_USER_ROUND)
                            }

                            a href=(format!("/user/{}", user.id)) {
                                minty-icon { (icon::CIRCLE_USER_ROUND) }
                                minty-title { (user.name) }
                            }

                            button hx-delete="/user/session" {
                                minty-icon { (icon::LOG_OUT) }
                                minty-title { "Sign Out" }
                            }
                        }
                    } @else {
                        a href="/signin" { (icon::LOG_IN) }
                    }

                    button { (icon::SETTINGS) }
                }
            }

            main { (self.page.full()) }
        }
    }
}
