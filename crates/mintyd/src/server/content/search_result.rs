use super::{icon, AsRender, Html, Progress};

use maud::{html, Markup, Render};
use minty::{http::query::QueryParams, PostPreview, PostQuery, Query};
use serde::{Serialize, Serializer};
use std::borrow::Cow;

#[derive(Debug)]
struct SearchResult<Q, T> {
    path: &'static str,
    query: Q,
    result: minty::SearchResult<T>,
}

impl<Q, T> SearchResult<Q, T>
where
    Q: Clone + Query + QueryParams,
{
    fn endpoint(&self, query: Q) -> String {
        format!("{}?{}", self.path, query.into_query_params())
    }

    fn more(&self) -> Option<String> {
        let mut pagination = self.query.pagination();
        pagination.from += self.result.hits.len() as u32;

        if pagination.from < self.result.total {
            pagination.size = 100;
            let mut query = self.query.clone();
            query.set_pagination(pagination);

            Some(self.endpoint(query))
        } else {
            None
        }
    }

    fn progress(&self, endpoint: &str) -> Markup {
        html! {
            div
                .padding
                .flex
                .center
                .secondary
                hx-get=(endpoint)
                hx-trigger="revealed"
                hx-swap="outerHTML"
            {
                (Progress)
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

impl<Q, T> Render for SearchResult<Q, T>
where
    T: AsRender,
    Q: Clone + Query + QueryParams,
{
    fn render(&self) -> Markup {
        html! {
            @for hit in &self.result.hits {
                (hit.as_render())
            }

            @if let Some(endpoint) = self.more() {
                (self.progress(&endpoint))
            }
        }
    }
}

#[derive(Debug)]
pub struct PostSearchResult(SearchResult<PostQuery, PostPreview>);

impl PostSearchResult {
    pub fn new(
        query: minty::PostQuery,
        result: minty::SearchResult<PostPreview>,
    ) -> Self {
        Self(SearchResult {
            path: "/posts",
            query,
            result,
        })
    }

    fn controls(&self) -> Markup {
        html! {
            form
                hx-get=(self.0.path)
                hx-trigger=
                    "submit, change find minty-select, change find minty-switch"
                hx-target="#post-search-results"
                .search-controls
                .flex-row
                .center
            {
                .search-field {
                    (icon::MAGNIFYING_GLASS.inline())

                    input
                        type="text"
                        name="q"
                        value=[self.0.query.text()]
                        placeholder="Search for posts..."
                        tab-index="0";
                }

                minty-select name="vis" {
                    minty-option value="public" {
                        minty-icon { (icon::EYE) }
                        minty-title { "Public" }
                    }

                    minty-option value="draft" {
                        minty-icon { (icon::SQUARE_PEN) }
                        minty-title { "Drafts" }
                    }
                }

                minty-select name="sort" {
                    minty-option value="created.desc" {
                        minty-icon { (icon::CLOCK_ARROW_DOWN) }
                        minty-title { "Newest" }
                    }
                    minty-option value="created.asc" {
                        minty-icon { (icon::CLOCK_ARROW_UP) }
                        minty-title { "Oldest" }
                    }
                    minty-option value="modified.desc" {
                        minty-icon { (icon::PENCIL) }
                        minty-title { "Latest Modified" }
                    }
                    minty-option value="modified.asc" {
                        minty-icon { (icon::PENCIL) }
                        minty-title { "Earliest Modified" }
                    }
                    minty-option value="title.asc" {
                        minty-icon { (icon::ARROW_DOWN_A_Z) }
                        minty-title { "A-Z" }
                    }
                    minty-option value="title.desc" {
                        minty-icon { (icon::ARROW_DOWN_Z_A) }
                        minty-title { "Z-A" }
                    }
                    minty-option value="relevance.desc" {
                        minty-icon { (icon::ARROW_DOWN_WIDE_NARROW) }
                        minty-title { "Most Relevant" }
                    }
                    minty-option value="relevance.asc" {
                        minty-icon { (icon::ARROW_DOWN_NARROW_WIDE) }
                        minty-title { "Least Relevance" }
                    }
                }

                button { (icon::ROTATE_CW) }
            }
        }
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
    fn page_title(&self) -> Cow<str> {
        "post search".into()
    }

    fn fragment(&self) -> Markup {
        self.0.render()
    }

    fn full(&self) -> Markup {
        html! {
            (self.controls())

            #post-search-results .grid {
                (self.0)
            }
        }
    }
}
