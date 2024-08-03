use super::icon;

use maud::{html, Markup, Render};

#[derive(Debug)]
pub struct ObjectPreview(pub minty::ObjectPreview);

impl ObjectPreview {
    fn preview(&self) -> Option<String> {
        self.0.preview_id.map(|id| format!("/object/{id}/data"))
    }

    fn file_type(&self) -> String {
        self.0.r#type.to_uppercase()
    }
}

impl Render for ObjectPreview {
    fn render(&self) -> Markup {
        html! {
            @if let Some(preview) = self.preview() {
                img .max-width-full src=(preview);
            } @else {
                .flex .center {
                    .object-placeholder { (icon::FILE_FILL) }
                    .object-placeholder-text
                    .absolute
                    .background-color
                    .bold
                    .z1 {
                        (self.file_type())
                    }
                }
            }
        }
    }
}
