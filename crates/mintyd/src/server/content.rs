mod comment;
mod css;
mod date_time;
mod home;
mod icon;
mod label;
mod navbar;
mod object_grid;
mod object_preview;
mod post;
mod post_preview;
mod script;
mod search_result;
mod source;
mod space;
mod tag;
mod templates;
mod user;
mod user_preview;
mod view;

pub use home::Home;
pub use post::Post;
pub use search_result::*;
pub use tag::Tag;
pub use user::User;
pub use user_preview::UserPreview;

use comment::Comments;
use css::Css;
use date_time::DateTime;
use icon::Icon;
use label::*;
use navbar::Navbar;
use object_grid::ObjectGrid;
use object_preview::ObjectPreview;
use post_preview::PostPreview;
use script::Script;
use source::*;
use space::Space;
use templates::Templates;
use view::*;

use super::accept::Accept;

use axum::{
    response::{IntoResponse, Response},
    Json,
};
use maud::{html, Markup, Render, DOCTYPE};
use serde::Serialize;

pub trait Html {
    fn page_title(&self) -> &str;

    fn full(&self) -> Markup;

    fn fragment(&self) -> Markup {
        self.full()
    }
}

trait AsRender {
    fn as_render(&self) -> impl Render;
}

pub struct Content<T> {
    pub accept: Accept,
    pub data: T,
}

impl<T> Content<T>
where
    T: Html,
{
    fn page(self) -> Markup {
        html! {
            (DOCTYPE)
            html {
                head {
                    title { (self.data.page_title()) }
                    (Css("/assets/index.css"))
                    (Script("/assets/index.js"))
                    (Templates)
                }

                body {
                    (Navbar::new(&self.data))
                }
            }
        }
    }

    fn full(self) -> Markup {
        self.data.full()
    }

    fn fragment(self) -> Markup {
        self.data.fragment()
    }
}

impl<T> Content<T>
where
    T: Serialize,
{
    fn json(self) -> Json<T> {
        Json(self.data)
    }
}

impl<T> IntoResponse for Content<T>
where
    T: Html + Serialize,
{
    fn into_response(self) -> Response {
        match self.accept {
            Accept::Html => self.page().into_response(),
            Accept::Boosted => self.full().into_response(),
            Accept::Fragment => self.fragment().into_response(),
            Accept::Json => self.json().into_response(),
        }
    }
}
