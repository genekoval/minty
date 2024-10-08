use super::style;

use maud::{html, Markup, Render};

pub struct MintyRange;

impl Render for MintyRange {
    fn render(&self) -> Markup {
        html! {
            template #minty-range-template {
                (style!("minty_range.css"))

                #track .track {}
                #fill .track {}
                #thumb {}
            }
        }
    }
}
