use super::icon;

use maud::{Markup, Render};

pub struct Progress;

impl Render for Progress {
    fn render(&self) -> Markup {
        icon::DOTS_6_ROTATE.render()
    }
}
