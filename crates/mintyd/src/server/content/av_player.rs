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

        html! {
            minty-audio autoplay src=(item.data_path()) {
                #track-info {
                    @if let Some(preview) = &self.post.preview {
                        (ObjectPreview::new(preview).rounded_corners())
                    }

                    div {
                        a href=(format!("/post/{}", self.post.id)) .block {
                            (self.title())
                        }
                    }
                }
            }
        }
    }
}
