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
mod search;
mod search_result;
mod space;
mod user;
mod user_preview;
mod view;

pub use home::Home;
pub use search::PostSearchResult;
pub use user::User;
pub use user_preview::UserPreview;

use css::Css;
use date_time::DateTime;
use icon::Icon;
use label::Label;
use navbar::Navbar;
use object_grid::ObjectGrid;
use object_preview::ObjectPreview;
use post_preview::PostPreview;
use script::Script;
use search_result::SearchResult;
use space::Space;
use view::*;

use super::accept::Accept;

use axum::{
    response::{IntoResponse, Response},
    Json,
};
use maud::{html, Markup, Render, DOCTYPE};
use serde::Serialize;
use std::fmt::Debug;

pub trait PageTitle {
    fn page_title(&self) -> &str;
}

pub trait IntoPage: Sized {
    type View: Debug + From<Self> + Html + PageTitle;

    fn into_page(self) -> Self::View {
        self.into()
    }
}

pub trait Html {
    fn full(&self) -> Markup;

    fn fragment(&self) -> Markup;
}

impl<T> Html for T
where
    T: Render,
{
    fn fragment(&self) -> Markup {
        self.render()
    }

    fn full(&self) -> Markup {
        self.render()
    }
}

pub struct Content<T> {
    pub accept: Accept,
    pub data: T,
}

impl<T> Content<T>
where
    T: IntoPage,
{
    fn page(self) -> Markup {
        let page = self.data.into_page();

        html! {
            (DOCTYPE)
            html {
                head {
                    title { (page.page_title()) }
                    (Css("/assets/styles.css"))
                    (Script("/assets/scripts/htmx-2.0.1.min.js"))
                }
                body {
                    (Navbar::new(&page))
                }
            }
        }
    }

    fn full(self) -> Markup {
        self.data.into_page().full()
    }

    fn fragment(self) -> Markup {
        self.data.into_page().fragment()
    }
}

impl<T: Serialize> Content<T> {
    fn json(self) -> Json<T> {
        Json(self.data)
    }
}

impl<T> IntoResponse for Content<T>
where
    T: IntoPage + Serialize,
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
