use super::{icon, Label, LabelIcon};

use maud::{html, Markup, Render};

pub struct Source<'a>(pub &'a minty::Source);

impl<'a> Render for Source<'a> {
    fn render(&self) -> Markup {
        html! {
            a href=(self.0.url) target="_blank" .fit-content .hover-underline {
                (Label::new(
                    self.0.url.to_string(),
                    self.0
                        .icon
                        .map(LabelIcon::Object)
                        .unwrap_or(icon::LINK.into())
                ))
            }
        }
    }
}

pub struct SourceList<'a>(pub &'a [minty::Source]);

impl<'a> Render for SourceList<'a> {
    fn render(&self) -> Markup {
        html! {
            @if !self.0.is_empty() {
                .flex-column .gap-p25em {
                    @for source in self.0 {
                        (Source(source))
                    }
                }
            }
        }
    }
}
