use super::ObjectPreview;

use crate::server::query::ObjectViewer;

use maud::{html, Markup, Render};
use minty::{http::ObjectExt, Uuid};

fn is_supported(ty: &str) -> bool {
    ["audio", "image"].contains(&ty)
}

fn script(ty: &str) -> Option<&'static str> {
    match ty {
        "audio" => {
            Some("on click add .with-player to body then show #av-player")
        }
        _ => None,
    }
}

fn swap(ty: &str) -> Option<&'static str> {
    match ty {
        "image" => Some("beforeend"),
        _ => None,
    }
}

fn target(ty: &str) -> &'static str {
    match ty {
        "audio" => "#av-player",
        _ => "body",
    }
}

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
                        @let ty = object.r#type.as_str();

                        @if is_supported(ty) {
                            @let link = format!(
                                "/post/{}/objects?{}",
                                self.post,
                                ObjectViewer { index }
                            );

                            a href=(object.data_path())
                                hx-get=(link)
                                hx-trigger="click"
                                hx-target=(target(ty))
                                hx-swap=[swap(ty)]
                                _=[script(ty)]
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
