use super::{
    color::PURPLE, icon, DateTime, Html, Label, PostSearchResult, SourceList,
    View,
};

use maud::{html, Markup, Render};
use serde::{Serialize, Serializer};

#[derive(Debug)]
pub struct User {
    pub user: minty::User,
    pub posts: Option<PostSearchResult>,
}

impl User {
    fn admin(&self) -> Option<impl Render> {
        self.user
            .admin
            .then(|| Label::icon("Admin", icon::BADGE_CHECK).color(PURPLE))
    }

    fn email(&self) -> impl Render + '_ {
        Label::icon(&self.user.email, icon::ENVELOPE)
    }

    fn created(&self) -> impl Render {
        DateTime::new(self.user.profile.created)
            .icon(icon::CALENDAR)
            .prefix("Joined")
    }

    fn comment_count(&self) -> impl Render {
        let count = format!(
            "{} Comment{}",
            self.user.comment_count,
            match self.user.comment_count {
                1 => "",
                _ => "s",
            }
        );

        Label::icon(count, icon::COMMENT)
    }

    fn post_count(&self) -> impl Render {
        let count = format!(
            "{} Post{}",
            self.user.post_count,
            match self.user.post_count {
                1 => "",
                _ => "s",
            }
        );

        Label::icon(count, icon::FILE_IMAGE)
    }

    fn tag_count(&self) -> impl Render {
        let count = format!(
            "{} Tag{}",
            self.user.tag_count,
            match self.user.tag_count {
                1 => "",
                _ => "s",
            }
        );

        Label::icon(count, icon::HASH)
    }
}

impl Serialize for User {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.user.serialize(serializer)
    }
}

impl Html for User {
    fn page_title(&self) -> &str {
        &self.user.profile.name
    }

    fn full(&self) -> Markup {
        html! {
            h1 { (self.user.profile.name) }

            @if !self.user.profile.aliases.is_empty() {
                ul .bold {
                    @for alias in &self.user.profile.aliases {
                        li { (alias) }
                    }
                }
            }

            @if !self.user.profile.description.is_empty() {
                p { (self.user.profile.description) }
            }

            (SourceList(&self.user.profile.sources))

            .flex-column
            .gap-p5em
            .font-smaller
            .secondary
            .margin-top
            .margin-bottom
            {
                @if let Some(admin) = self.admin() {
                    (admin)
                }

                (self.email())
                (self.created())
                (self.post_count())
                (self.comment_count())
                (self.tag_count())
            }

            @if let Some(posts) = &self.posts {
                (posts.full())
            }
        }
    }
}
