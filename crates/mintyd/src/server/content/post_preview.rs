use super::{
    icon, AsRender, DateTime, Label, NavigationLink, ObjectPreview,
    UserPreview, View,
};

use maud::{html, Markup, Render};

#[derive(Debug)]
pub struct PostPreview<'a>(pub &'a minty::PostPreview);

impl<'a> PostPreview<'a> {
    fn created(&self) -> impl Render {
        DateTime::new(self.0.created).icon(icon::CLOCK).abbrev()
    }

    fn object_count(&self) -> Markup {
        html! {
            span {
                (Label::icon(self.0.object_count.to_string(), icon::FILE))
            }
        }
    }

    fn comment_count(&self) -> Markup {
        html! {
            span {
                (Label::icon(self.0.comment_count.to_string(), icon::COMMENT))
            }
        }
    }

    fn content(&self) -> Markup {
        html! {
            .post-preview-image .secondary {
                @if let Some(preview) = self.preview() {
                    (preview)
                } @else {
                    (icon::ALIGN_LEFT)
                }
            }

            .flex-row .font-smaller .padding .gap-2 .secondary {
                (UserPreview::new(self.0.poster.as_ref()).as_label())
                (self.created())
                (self.object_count())
                (self.comment_count())
            }

            span
                .font-larger
                .text-color
                .padding-leading
                .padding-trailing
            {
                (self.0.title)
            }
        }
    }

    fn path(&self) -> String {
        format!("/post/{}", self.0.id)
    }

    fn preview(&self) -> Option<impl Render + '_> {
        self.0
            .preview
            .as_ref()
            .map(|preview| ObjectPreview::new(preview).rounded_corners())
    }
}

impl<'a> From<&'a minty::PostPreview> for PostPreview<'a> {
    fn from(value: &'a minty::PostPreview) -> Self {
        Self(value)
    }
}

impl<'a> Render for PostPreview<'a> {
    fn render(&self) -> Markup {
        NavigationLink::new(self.path(), self.content())
            .class("post-preview")
            .render()
    }
}

impl AsRender for minty::PostPreview {
    fn as_render(&self) -> impl Render {
        PostPreview(self)
    }
}
