use super::ObjectPreview;

use maud::{html, Markup, Render};

#[derive(Debug)]
pub struct ObjectGrid(Vec<ObjectPreview>);

impl ObjectGrid {
    pub fn new(objects: Vec<minty::ObjectPreview>) -> Self {
        Self(objects.into_iter().map(Into::into).collect())
    }
}

impl Render for ObjectGrid {
    fn render(&self) -> Markup {
        html! {
            @if !self.0.is_empty() {
                .object-grid {
                    @for object in &self.0 {
                        (object)
                    }
                }
            }
        }
    }
}
