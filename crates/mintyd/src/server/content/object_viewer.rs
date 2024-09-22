use super::{AvPlayer, Html, ImageViewer};

use maud::{html, Markup, Render};
use minty::{ObjectPreview, PostPreview};
use serde::Serialize;
use std::borrow::Cow;

#[derive(Debug, Serialize)]
pub struct ObjectViewer {
    post: PostPreview,
    objects: Vec<ObjectPreview>,
    index: usize,
}

impl ObjectViewer {
    pub fn new(
        post: PostPreview,
        objects: Vec<ObjectPreview>,
        index: usize,
    ) -> Self {
        let object = objects.get(index).expect("index should be valid");

        let id = object.id;
        let ty = object.r#type.clone();

        let objects: Vec<_> = objects
            .into_iter()
            .filter(|object| object.r#type == ty)
            .collect();

        let index = objects.iter().position(|object| object.id == id).unwrap();

        Self {
            post,
            objects,
            index,
        }
    }
}

impl Html for ObjectViewer {
    fn page_title(&self) -> Cow<str> {
        let len = self.objects.len();

        format!(
            "{} object{}",
            len,
            match len {
                1 => "",
                _ => "s",
            }
        )
        .into()
    }

    fn full(&self) -> Markup {
        let Some(first) = self.objects.first() else {
            return html! {
                h1 { "No objects to display" }
            };
        };

        let ty = first.r#type.as_str();

        match ty {
            "audio" => AvPlayer {
                post: &self.post,
                items: &self.objects,
                index: self.index,
            }
            .render(),
            "image" => ImageViewer::new(&self.objects, self.index).render(),
            _ => html! {
                h1 { (format!("Objects of type '{ty}' are not supported.")) }
            },
        }
    }
}
