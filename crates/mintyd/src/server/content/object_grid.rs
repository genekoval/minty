use super::ObjectPreview;

use crate::server::query::ObjectViewer;

use maud::{html, Markup, Render};
use minty::{http::ObjectExt, Uuid};

#[derive(Debug)]
pub struct ObjectGrid<'a> {
    pub post: Uuid,
    pub objects: &'a [minty::ObjectPreview],
}

impl<'a> Render for ObjectGrid<'a> {
    fn render(&self) -> Markup {
        html! {
            @if !self.objects.is_empty() {
                .object-grid {
                    @for (index, object) in self
                        .objects
                        .iter()
                        .enumerate()
                    {
                        @if object.r#type == "image" {
                            @let link = format!(
                                "/post/{}/objects?{}",
                                self.post,
                                ObjectViewer { index }
                            );

                            a href=(object.data_path())
                                hx-get=(link)
                                hx-trigger="click"
                                hx-target="body"
                                hx-swap="beforeend"
                            {
                                (ObjectPreview::new(object))
                            }
                        } @else {
                            a href=(object.data_path()) target="_blank" {
                                (ObjectPreview::new(object))
                            }
                        }
                    }
                }
            }
        }
    }
}
