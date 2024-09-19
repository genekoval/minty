use super::Html;

use maud::{html, Markup};
use serde::Serialize;
use std::borrow::Cow;

#[derive(Debug, Serialize)]
pub struct SignIn;

impl Html for SignIn {
    fn page_title(&self) -> Cow<str> {
        "Sign In".into()
    }

    fn display_navigation(&self) -> bool {
        false
    }

    fn full(&self) -> Markup {
        html! {
            .full-page .flex-column .center {
                h1 { "Welcome to Minty" }

                form method="post" .flex-column .center .gap-2 {
                    div .flex-column .gap-p5em {
                        label for="email" { "Email" }

                        .text-field {
                            input
                                #email
                                type="email"
                                name="email"
                                required;
                        }
                    }

                    div .flex-column .gap-p25em {
                        label for="password" { "Password" }

                        .text-field {
                            input
                                #password
                                type="password"
                                name="password"
                                required
                                minlength="8"
                                autocomplete="current-password";
                        }
                    }

                    button .submit-button { "Sign In" }
                }
            }
        }
    }
}
