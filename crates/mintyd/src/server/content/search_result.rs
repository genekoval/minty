use super::{icon, AsRender, Html};

use maud::{html, Markup, Render};
use minty::{http::query::QueryParams, PostPreview, PostQuery, Query};
use serde::{Serialize, Serializer};

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
            path: "posts",
            query,
            result,
        })
    }

    fn controls(&self) -> Markup {
        html! {
            form
                hx-get=(self.0.path)
                hx-trigger="submit, change find select"
                hx-target="#post-search-results"
                .search-controls
                .flex-row
                .center
            {
                .search-field {
                    input
                        type="text"
                        name="q"
                        value=[self.0.query.text()]
                        placeholder="Search for posts..."
                        tab-index="0";
                }

                select name="sort" {
                    option value="created" { "Created" }
                    option value="modified" { "Modified" }
                    option value="title" { "Title" }
                    option value="relevance" { "Relevance" }
                }

                button { (icon::MAGNIFYING_GLASS) }
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
    fn page_title(&self) -> &str {
        "post search"
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
