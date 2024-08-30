use super::{
    icon, Comments, DateTime, Html, Label, ObjectGrid, PostBanner, PostPreview,
    UserPreview,
};

use maud::{html, Markup, Render};
use minty::{CommentData, Visibility};
use serde::{Serialize, Serializer};

#[derive(Debug)]
pub struct Post {
    pub post: minty::Post,
    pub comments: Vec<CommentData>,
}

impl Post {
    fn title(&self) -> Markup {
        html! {
            @if !self.post.title.is_empty() {
                h1 { (self.post.title) }
            }
        }
    }

    fn description(&self) -> Markup {
        html! {
            @if !self.post.description.is_empty() {
                p { (self.post.description) }
            }
        }
    }

    fn poster(&self) -> impl Render + '_ {
        UserPreview::new(self.post.poster.as_ref())
    }

    fn created(&self) -> impl Render {
        DateTime::new(self.post.created)
            .icon(icon::CLOCK)
            .prefix("Posted")
    }

    fn modified(&self) -> Option<impl Render> {
        (self.post.modified != self.post.created).then(|| {
            DateTime::new(self.post.modified)
                .icon(icon::PENCIL)
                .prefix("Last modified")
        })
    }

    fn metadata(&self) -> Markup {
        html! {
            div .font-smaller ."leading-1-5" .secondary {
                div { (self.poster()) }

                div { (self.created()) }

                @if let Some(modified) = self.modified() {
                    div { (modified) }
                }
            }
        }
    }

    fn objects(&self) -> impl Render + '_ {
        ObjectGrid(&self.post.objects)
    }

    fn posts(&self) -> Markup {
        html! {
            @if !self.post.posts.is_empty() {
                .flex-column .margin-top {
                    @for post in &self.post.posts {
                        (PostPreview(post))
                    }
                }
            }
        }
    }

    fn tags(&self) -> Markup {
        html! {
            @if !self.post.tags.is_empty() {
                .tags .flex-row .flex-wrap {
                    @for tag in &self.post.tags {
                        a href=(format!("/tag/{}", tag.id)) {
                            (Label::icon(&tag.name, icon::HASH))
                        }
                    }
                }
            }
        }
    }
}

impl Serialize for Post {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.post.serialize(serializer)
    }
}

impl Html for Post {
    fn page_title(&self) -> &str {
        let title = self.post.title.as_str();

        if title.is_empty() {
            "Untitled"
        } else {
            title
        }
    }

    fn full(&self) -> Markup {
        html! {
            article #post {
                @if self.post.visibility == Visibility::Draft {
                    (PostBanner {
                        post: &self.post,
                        is_editing: false
                    })
                }

                (self.title())
                (self.metadata())
                (self.description())
                (self.objects())
                (self.posts())
                (self.tags())
            }

            (Comments(&self.comments))
        }
    }
}
