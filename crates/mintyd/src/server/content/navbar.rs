use super::{icon, Label, NavigationLink};

use maud::{html, Markup, Render};
use minty::UserPreview;

#[derive(Debug)]
pub struct Navbar {
    pub user: Option<UserPreview>,
}

impl Render for Navbar {
    fn render(&self) -> Markup {
        html! {
            nav .flex-column {
                .nav-primary .nav-section {
                    (NavigationLink::new("/", icon::HOME))

                    minty-menu {
                        span slot="menu-button" { (icon::PLUS) }

                        (NavigationLink::new(
                            "/post",
                            Label::icon("New Post", icon::FILE_IMAGE)
                        ))
                    }
                }

                .nav-secondary .nav-section {
                    @if let Some(user) = &self.user {
                        minty-menu {
                            span slot="menu-button" {
                                (icon::CIRCLE_USER_ROUND)
                            }

                            (NavigationLink::new(
                                format!("/user/{}", user.id),
                                Label::icon(
                                    &user.name,
                                    icon::CIRCLE_USER_ROUND,
                                ),
                            ))

                            (NavigationLink::new(
                                "/posts?vis=draft",
                                Label::icon("Drafts", icon::SQUARE_PEN),
                            ))

                            button hx-delete="/user/session" {
                                (Label::icon("Sign Out", icon::LOG_OUT))
                            }
                        }
                    } @else {
                        a href="/signin" { (icon::LOG_IN) }
                    }

                    button { (icon::SETTINGS) }
                }
            }

        }
    }
}
