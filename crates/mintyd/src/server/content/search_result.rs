use super::{icon, Html, PageTitle};

use maud::{html, Markup, Render};
use minty::Query;
use serde::Serialize;

#[derive(Debug)]
pub struct SearchResult<T, Q> {
    endpoint: &'static str,
    query: Q,
    hits: Vec<T>,
    more: bool,
}

impl<T: Render, Q: Serialize> SearchResult<T, Q> {
    pub fn new<U, R>(
        endpoint: &'static str,
        mut query: R,
        result: minty::SearchResult<U>,
    ) -> Self
    where
        T: From<U>,
        R: Into<Q> + Query,
    {
        let mut pagination = query.pagination();

        pagination.from += result.hits.len() as u32;
        pagination.size = 100;

        query.set_pagination(pagination);

        Self {
            endpoint,
            query: query.into(),
            hits: result.hits.into_iter().map(Into::into).collect(),
            more: pagination.from < result.total,
        }
    }

    fn path(&self) -> String {
        let query = serde_urlencoded::to_string(&self.query)
            .expect("query serialization should always succeed");

        format!("/{}?{query}", self.endpoint)
    }

    fn progress(&self) -> Markup {
        html! {
            div
                .padding
                .flex
                .center
                .secondary
                hx-get=(self.path())
                hx-trigger="revealed"
                hx-swap="outerHTML"
            {
                (icon::DOTS_6_ROTATE)
            }
        }
    }
}

impl<T: Render, Q: Serialize> Html for SearchResult<T, Q> {
    fn fragment(&self) -> Markup {
        html! {
            @for hit in &self.hits {
                (hit)
            }

            @if self.more {
                (self.progress())
            }
        }
    }

    fn full(&self) -> Markup {
        html! {
            .grid {
                (self.fragment())
            }
        }
    }
}

impl<T, Q> PageTitle for SearchResult<T, Q> {
    fn page_title(&self) -> &str {
        "search results"
    }
}
