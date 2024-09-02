use super::ObjectPreview;

use maud::{html, Markup, Render};
use minty::http::ObjectExt;

#[derive(Debug)]
pub struct ObjectGrid<'a>(pub &'a [minty::ObjectPreview]);

impl<'a> Render for ObjectGrid<'a> {
    fn render(&self) -> Markup {
        html! {
            @if !self.0.is_empty() {
                .object-grid {
                    @for object in self.0 {
                        a href=(object.data_path())
                            hx-get=(format!("/object/{}", object.id))
                            hx-trigger="click"
                            hx-target="body"
                            hx-swap="beforeend"
                        {
                            (ObjectPreview::new(object))
                        }
                    }
                }
            }
        }
    }
}
