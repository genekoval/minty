use super::ObjectPreview;

use maud::{html, Markup, Render};

#[derive(Debug)]
pub struct ObjectGrid<'a>(pub &'a [minty::ObjectPreview]);

impl<'a> Render for ObjectGrid<'a> {
    fn render(&self) -> Markup {
        html! {
            @if !self.0.is_empty() {
                .object-grid {
                    @for object in self.0 {
                        (ObjectPreview::new(object))
                    }
                }
            }
        }
    }
}
