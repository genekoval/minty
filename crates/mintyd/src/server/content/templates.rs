macro_rules! style {
    ($file:literal) => {
        ::maud::html! {
            style {
                (::maud::PreEscaped(include_str!($file)))
            }
        }
    };
}

use style;

mod minty_audio;
mod minty_menu;
mod minty_range;
mod minty_select;

use minty_audio::MintyAudio;
use minty_menu::MintyMenu;
use minty_range::MintyRange;
use minty_select::MintySelect;

use maud::{html, Markup, Render};

pub struct Templates;

impl Render for Templates {
    fn render(&self) -> Markup {
        html! {
            (MintyAudio)
            (MintyMenu)
            (MintyRange)
            (MintySelect)
        }
    }
}
