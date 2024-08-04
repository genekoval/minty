use maud::{html, Markup, Render};

#[derive(Debug)]
pub struct List<T>(Vec<T>);

impl<T, U> From<Vec<T>> for List<U>
where
    U: From<T>,
{
    fn from(value: Vec<T>) -> Self {
        Self(value.into_iter().map(Into::into).collect())
    }
}

impl<T: Render> Render for List<T> {
    fn render(&self) -> Markup {
        html! {
            @if !self.0.is_empty() {
                .grid {
                    @for item in &self.0 {
                        (item)
                    }
                }
            }
        }
    }
}
