use super::ObjectPreview;

use maud::{html, Markup, Render};
use minty::{http::ObjectExt, PostPreview};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AvPlayer<'a> {
    pub post: &'a PostPreview,
    pub items: &'a [minty::ObjectPreview],
    pub index: usize,
}

impl<'a> AvPlayer<'a> {
    fn title(&self) -> &str {
        if self.post.title.is_empty() {
            "Untitled"
        } else {
            &self.post.title
        }
    }
}

impl<'a> Render for AvPlayer<'a> {
    fn render(&self) -> Markup {
        let item = self.items.get(self.index).unwrap();
        let post = format!("/post/{}", self.post.id);

        html! {
            minty-audio autoplay src=(item.data_path()) {
                #track-info {
                    (ObjectPreview::new(item).rounded_corners())

                    div {
                        a href=(post)
                            hx-get=(post)
                            hx-trigger="click"
                            hx-target="main"
                        {
                            (self.title())
                        }
                    }
                }
            }
        }
    }
}
