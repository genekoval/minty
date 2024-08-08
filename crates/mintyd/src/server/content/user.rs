use crate::server::content::Html;

use super::{
    color::PURPLE, icon, DateTime, IntoPage, Label, PageTitle,
    PostSearchResult, View,
};

use maud::{html, Markup, Render};
use serde::{Serialize, Serializer};

#[derive(Debug)]
pub struct User {
    user: minty::User,
    posts: Option<PostSearchResult>,
}

impl User {
    pub fn new(user: minty::User, posts: Option<PostSearchResult>) -> Self {
        Self { user, posts }
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

impl IntoPage for User {
    type View = UserView;
}

#[derive(Debug)]
pub struct UserView {
    name: String,
    aliases: Vec<String>,
    description: String,
    created: DateTime,
    email: String,
    admin: bool,
    post_count: u32,
    comment_count: u32,
    tag_count: u32,
    posts: Option<<PostSearchResult as IntoPage>::View>,
}

impl UserView {
    fn comment_count(&self) -> String {
        format!(
            "{} Comment{}",
            self.comment_count,
            match self.comment_count {
                1 => "",
                _ => "s",
            }
        )
    }

    fn post_count(&self) -> String {
        format!(
            "{} Post{}",
            self.post_count,
            match self.post_count {
                1 => "",
                _ => "s",
            }
        )
    }

    fn tag_count(&self) -> String {
        format!(
            "{} Tag{}",
            self.tag_count,
            match self.tag_count {
                1 => "",
                _ => "s",
            }
        )
    }
}

impl From<User> for UserView {
    fn from(value: User) -> Self {
        let user = value.user;
        let profile = user.profile;

        Self {
            name: profile.name,
            aliases: profile.aliases,
            description: profile.description,
            created: DateTime::new(profile.created)
                .icon(icon::CALENDAR)
                .prefix("Joined"),
            email: user.email,
            admin: user.admin,
            post_count: user.post_count,
            comment_count: user.comment_count,
            tag_count: user.tag_count,
            posts: value.posts.map(IntoPage::into_page),
        }
    }
}

impl Render for UserView {
    fn render(&self) -> Markup {
        html! {
            h1 { (self.name) }

            ul .bold {
                @for alias in &self.aliases {
                    li { (alias) }
                }
            }

            @if !self.description.is_empty() {
                p { (self.description) }
            }

            .flex-column
            .gap-p5em
            .font-smaller
            .secondary
            .margin-top
            .margin-bottom
            {
                @if self.admin {
                    (Label::new("Admin", icon::BADGE_CHECK).color(PURPLE))
                }

                (Label::new(&self.email, icon::ENVELOPE))

                (self.created)

                (Label::new(self.post_count(), icon::FILE_IMAGE))
                (Label::new(self.comment_count(), icon::COMMENT))
                (Label::new(self.tag_count(), icon::HASH))
            }

            @if let Some(posts) = &self.posts {
                (posts.full())
            }
        }
    }
}

impl PageTitle for UserView {
    fn page_title(&self) -> &str {
        &self.name
    }
}
