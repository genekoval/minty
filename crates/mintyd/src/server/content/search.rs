use super::{IntoPage, PostPreview, SearchResult};

use minty::{http::query, PostQuery};
use serde::{Serialize, Serializer};

#[derive(Debug)]
pub struct PostSearchResult {
    pub query: PostQuery,
    pub result: minty::SearchResult<minty::PostPreview>,
}

impl From<PostSearchResult> for SearchResult<PostPreview, query::PostQuery> {
    fn from(value: PostSearchResult) -> Self {
        Self::new("posts", value.query, value.result)
    }
}

impl Serialize for PostSearchResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.result.serialize(serializer)
    }
}

impl IntoPage for PostSearchResult {
    type View = SearchResult<PostPreview, query::PostQuery>;
}
