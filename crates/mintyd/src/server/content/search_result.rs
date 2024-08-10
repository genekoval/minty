use super::{icon, AsRender, Html};

use maud::{html, Markup, Render};
use minty::{http::query, PostPreview, Query};
use serde::{Serialize, Serializer};

#[derive(Debug)]
struct SearchResult<Q, T> {
    endpoint: &'static str,
    query: Q,
    result: minty::SearchResult<T>,
    more: bool,
}

impl<Q, T> SearchResult<Q, T>
where
    Q: Serialize,
{
    fn new<U>(
        endpoint: &'static str,
        mut query: U,
        result: minty::SearchResult<T>,
    ) -> Self
    where
        U: Into<Q> + Query,
    {
        let mut pagination = query.pagination();

        pagination.from += result.hits.len() as u32;
        pagination.size = 100;

        let more = pagination.from < result.total;

        query.set_pagination(pagination);

        Self {
            endpoint,
            query: query.into(),
            result,
            more,
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

impl<Q, T> Serialize for SearchResult<Q, T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.result.serialize(serializer)
    }
}

impl<Q, T> Html for SearchResult<Q, T>
where
    T: AsRender,
    Q: Serialize,
{
    fn page_title(&self) -> &str {
        "search results"
    }

    fn fragment(&self) -> Markup {
        html! {
            @for hit in &self.result.hits {
                (hit.as_render())
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

impl<Q, T> Render for SearchResult<Q, T>
where
    T: AsRender,
    Q: Serialize,
{
    fn render(&self) -> Markup {
        self.full()
    }
}

#[derive(Debug)]
pub struct PostSearchResult(SearchResult<query::PostQuery, PostPreview>);

impl PostSearchResult {
    pub fn new(
        query: minty::PostQuery,
        result: minty::SearchResult<PostPreview>,
    ) -> Self {
        Self(SearchResult::new("posts", query, result))
    }
}

impl Serialize for PostSearchResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl Html for PostSearchResult {
    fn page_title(&self) -> &str {
        self.0.page_title()
    }

    fn full(&self) -> Markup {
        self.0.full()
    }
}
