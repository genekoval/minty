use super::{IntoPage, PageTitle, PostPreview, SearchResult};

use maud::{Markup, Render};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Home(pub minty::SearchResult<minty::PostPreview>);

pub struct HomePage(SearchResult<PostPreview>);

impl From<Home> for HomePage {
    fn from(value: Home) -> Self {
        Self(value.0.into())
    }
}

impl Render for HomePage {
    fn render(&self) -> Markup {
        self.0.render()
    }
}

impl PageTitle for HomePage {
    fn page_title(&self) -> &str {
        "Minty"
    }
}

impl IntoPage for Home {
    type View = HomePage;
}
