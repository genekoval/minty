mod css;
mod post;

pub use post::Post;

use css::Css;

use super::accept::Accept;

use axum::{
    response::{IntoResponse, Response},
    Json,
};
use maud::{html, Markup, Render, DOCTYPE};
use serde::Serialize;
use std::fmt::Display;

pub struct Content<T> {
    pub accept: Accept,
    pub data: T,
}

impl<T> Content<T>
where
    T: Display + Render,
{
    fn html(&self) -> Markup {
        html! {
            (DOCTYPE)
            html {
                head {
                    title { (self.data.to_string()) }
                    (Css("/assets/styles.css"))
                }
                body {
                    (self.data)
                }
            }
        }
    }
}

impl<T: Serialize> Content<T> {
    fn json(self) -> Json<T> {
        Json(self.data)
    }
}

impl<T> IntoResponse for Content<T>
where
    T: Display + Render + Serialize,
{
    fn into_response(self) -> Response {
        match self.accept {
            Accept::Html => self.html().into_response(),
            Accept::Json => self.json().into_response(),
        }
    }
}
