mod css;
mod date_time;
mod icon;
mod post;
mod user_preview;

pub use user_preview::UserPreview;

use css::Css;
use date_time::DateTime;
use icon::Icon;

use super::accept::Accept;

use axum::{
    response::{IntoResponse, Response},
    Json,
};
use maud::{html, Markup, Render, DOCTYPE};
use serde::Serialize;

pub trait PageTitle {
    fn page_title(&self) -> &str;
}

pub trait IntoPage: Sized {
    type View: From<Self> + Render + PageTitle;

    fn into_page(self) -> Self::View {
        self.into()
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
    fn html(self) -> Markup {
        let page = self.data.into_page();

        html! {
            (DOCTYPE)
            html {
                head {
                    title { (page.page_title()) }
                    (Css("/assets/styles.css"))
                }
                body {
                    (page)
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
    T: IntoPage + Serialize,
{
    fn into_response(self) -> Response {
        match self.accept {
            Accept::Html => self.html().into_response(),
            Accept::Json => self.json().into_response(),
        }
    }
}
