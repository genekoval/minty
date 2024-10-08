use super::style;

use crate::server::content::icon;

use maud::{html, Markup, Render};

pub struct MintyVolume;

impl Render for MintyVolume {
    fn render(&self) -> Markup {
        html! {
            template #minty-volume-template {
                (style!("minty_volume.css"))

                button type="button" data-state="high" {
                    (icon::VOLUME_1)
                    (icon::VOLUME_2)
                    (icon::VOLUME_X)
                }

                minty-range value="1" min="0" max="1" {}
            }
        }
    }
}
