use super::List;

use maud::{html, Markup, Render};

pub struct SearchResult<T> {
    total: u32,
    hits: List<T>,
}

impl<T, U> From<minty::SearchResult<T>> for SearchResult<U>
where
    U: From<T>,
{
    fn from(value: minty::SearchResult<T>) -> Self {
        Self {
            total: value.total,
            hits: value.hits.into(),
        }
    }
}

impl<T: Render> Render for SearchResult<T> {
    fn render(&self) -> Markup {
        html! {
            (self.hits)
        }
    }
}
