use super::{
    Html, IntoPage, PageTitle, PostPreview, PostSearchResult, SearchResult,
};

use maud::{Markup, Render};
use minty::{http::query, PostQuery};
use serde::{Serialize, Serializer};

#[derive(Debug)]
pub struct Home(PostSearchResult);

impl Home {
    pub fn new(
        query: PostQuery,
        result: minty::SearchResult<minty::PostPreview>,
    ) -> Self {
        Self(PostSearchResult { query, result })
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

impl IntoPage for Home {
    type View = HomePage;
}

#[derive(Debug)]
pub struct HomePage(SearchResult<PostPreview, query::PostQuery>);

impl From<Home> for HomePage {
    fn from(value: Home) -> Self {
        Self(value.0.into())
    }
}

impl Render for HomePage {
    fn render(&self) -> Markup {
        self.0.full()
    }
}

impl PageTitle for HomePage {
    fn page_title(&self) -> &str {
        "Minty"
    }
}
