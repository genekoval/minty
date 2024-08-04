use super::{icon, DateTime, Label, ObjectPreview, UserPreview};

use maud::{html, Markup, Render};
use minty::Uuid;

#[derive(Debug)]
pub struct PostPreview {
    id: Uuid,
    poster: UserPreview,
    title: String,
    preview: Option<ObjectPreview>,
    comment_count: u32,
    object_count: u32,
    created: DateTime,
}

impl PostPreview {
    fn path(&self) -> String {
        format!("/post/{}", self.id)
    }

    fn object_count(&self) -> impl Render {
        Label::new(self.object_count.to_string(), icon::FILE)
    }

    fn comment_count(&self) -> impl Render {
        Label::new(self.comment_count.to_string(), icon::COMMENT)
    }
}

impl From<minty::PostPreview> for PostPreview {
    fn from(value: minty::PostPreview) -> Self {
        Self {
            id: value.id,
            poster: UserPreview::new(value.poster),
            title: value.title,
            preview: value
                .preview
                .map(|preview| ObjectPreview::new(preview).rounded_corners()),
            comment_count: value.comment_count,
            object_count: value.object_count,
            created: DateTime::new(value.created).icon(icon::CLOCK).abbrev(),
        }
    }
}

impl Render for PostPreview {
    fn render(&self) -> Markup {
        html! {
            a href=(self.path())
                .block
                .post-preview
                .divider
                .hover-highlight
            {
                .post-preview-image {
                    @if let Some(preview) = &self.preview {
                        (preview)
                    } @else {
                        (icon::ALIGN_LEFT)
                    }
                }

                .flex .font-smaller .padding .gap-2 {
                    (self.poster.as_label())
                    (self.created)
                    (self.object_count())
                    (self.comment_count())
                }

                span
                    .font-larger
                    .text-color
                    .padding-leading
                    .padding-trailing
                {
                    (self.title)
                }
            }
        }
    }
}
