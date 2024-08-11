use super::{icon, DateTime, UserPreview};

use core::slice::Iter;
use maud::{html, Markup, Render};
use minty::CommentData;
use std::iter::Peekable;

struct Comment<'a>(&'a CommentData);

impl<'a> Comment<'a> {
    fn created(&self) -> impl Render {
        DateTime::new(self.0.created).icon(icon::CLOCK).abbrev()
    }

    fn user(&self) -> impl Render + '_ {
        UserPreview::new(self.0.user.as_ref())
    }
}

impl<'a> Render for Comment<'a> {
    fn render(&self) -> Markup {
        html! {
            .comment {
                .comment-metadata .flex-row .gap-1 .font-smaller .secondary {
                    (self.user())
                    (self.created())
                }

                p .comment-content { (self.0.content) }
            }
        }
    }
}

fn render_threads(iter: &mut Peekable<Iter<CommentData>>) -> Markup {
    let comment = iter.next().unwrap();
    let next = iter.peek().map(|next| next.level);
    let id = format!("comment-{}", comment.id);

    html! {
        #(id) .comment-grid {
            (Comment(comment))

            @if next > Some(comment.level) {
                a href=(format!("#{id}")) .block .comment-indent {}

                .comment-replies {
                    (render_threads(iter))
                }
            }
        }

        @while iter.peek().is_some_and(|&next| next.level == comment.level) {
            (render_threads(iter))
        }
    }
}

pub struct Comments<'a>(pub &'a [CommentData]);

impl<'a> Render for Comments<'a> {
    fn render(&self) -> Markup {
        let mut iter = self.0.iter().peekable();

        html! {
            @if iter.peek().is_some() {
                .margin-top {
                    (render_threads(&mut iter))
                }
            }
        }
    }
}
