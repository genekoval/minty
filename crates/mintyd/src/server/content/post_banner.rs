use maud::{html, Markup, Render};
use minty::{Post, Visibility};

#[derive(Debug)]
pub struct PostBanner<'a> {
    pub post: &'a Post,
    pub is_editing: bool,
}

impl<'a> PostBanner<'a> {
    fn endpoint(&self) -> String {
        let id = self.post.id;
        let edit = if self.is_editing { "" } else { "/edit" };

        format!("/post/{id}{edit}")
    }
}

impl<'a> Render for PostBanner<'a> {
    fn render(&self) -> Markup {
        html! {
            .draft-banner {
                span .bold .secondary { "Draft" }

                span #saved .font-smaller {}

                button
                    hx-get=(self.endpoint())
                    hx-target="#post"
                    hx-swap="outerHTML"
                {
                    @if self.is_editing {
                        @if self.post.visibility == Visibility::Draft {
                            "Preview"
                        } @else {
                            "Done"
                        }
                    } @else {
                        "Edit"
                    }
                }
            }
        }
    }
}
