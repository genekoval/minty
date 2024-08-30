use crate::server::content::post_banner::PostBanner;

use super::Html;

use maud::{html, Markup};
use minty::Post;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PostEdit(pub Post);

impl PostEdit {
    fn endpoint(&self, path: &str) -> String {
        let id = self.0.id;
        format!("/post/{id}/{path}")
    }
}

impl Html for PostEdit {
    fn page_title(&self) -> &str {
        let title = self.0.title.as_str();

        if title.is_empty() {
            "Untitled"
        } else {
            title
        }
    }

    fn full(&self) -> Markup {
        html! {
            article #post {
                (PostBanner {
                    post: &self.0,
                    is_editing: true,
                })

                .labeled {
                    label for="title" { "Title" }
                }
                .text-field {
                    input
                        #title
                        type="text"
                        name="title"
                        placeholder="Untitled"
                        value=(self.0.title)
                        hx-put=(self.endpoint("title"))
                        hx-trigger="change"
                        hx-target="#saved";
                }

                .labeled {
                    label for="description" { "Description" }
                    textarea
                        #description
                        name="description"
                        rows="10"
                        hx-put=(self.endpoint("description"))
                        hx-trigger="change"
                        hx-target="#saved"
                    {
                        (self.0.description)
                    }
                }
            }
        }
    }
}
