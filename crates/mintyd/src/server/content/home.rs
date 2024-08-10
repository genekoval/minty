use super::{Html, PostSearchResult};

use maud::Markup;
use minty::{PostPreview, PostQuery, SearchResult};
use serde::{Serialize, Serializer};

#[derive(Debug)]
pub struct Home(PostSearchResult);

impl Home {
    pub fn new(query: PostQuery, result: SearchResult<PostPreview>) -> Self {
        Self(PostSearchResult::new(query, result))
    }
}

impl Serialize for Home {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl Html for Home {
    fn page_title(&self) -> &str {
        "Minty"
    }

    fn full(&self) -> Markup {
        self.0.full()
    }
}
