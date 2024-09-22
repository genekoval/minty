use super::{icon, ObjectPreview};

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
            .av-player-section {
                @if let Some(preview) = &self.post.preview {
                    (ObjectPreview::new(preview).rounded_corners())
                }

                a href=(format!("/post/{}", self.post.id)) .block .secondary {
                    (self.title())
                }
            }

            audio autoplay controls src=(item.data_path()) {}

            .av-player-section {
                button
                    .plain
                    _="on click trigger closePlayer"
                { (icon::X) }
            }
        }
    }
}
