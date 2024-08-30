use super::{DateTime, Html};

use maud::{html, Markup};
use serde::{Serialize, Serializer};

#[derive(Clone, Debug)]
pub struct SavedChanges {
    pub title: Option<String>,
    pub modified: minty::DateTime,
}

impl Serialize for SavedChanges {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.modified.serialize(serializer)
    }
}

impl Html for SavedChanges {
    fn page_title(&self) -> &str {
        self.title.as_deref().unwrap_or_default()
    }

    fn full(&self) -> Markup {
        html! {
            @if let Some(title) = &self.title {
                title {
                    @if title.is_empty() {
                        "Untitled"
                    } @else {
                        (title)
                    }
                }
            }

            span .secondary {
                (DateTime::new(self.modified).prefix("Changes saved"))
            }
        }
    }
}
