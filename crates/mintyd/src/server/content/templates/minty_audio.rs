use super::style;

use crate::server::content::icon;

use maud::{html, Markup, Render};

const TIME_PLACEHOLDER: &str = "-:--";

pub struct MintyAudio;

impl Render for MintyAudio {
    fn render(&self) -> Markup {
        html! {
            template #minty-audio-template {
                (style!("minty_audio.css"))

                audio {}

                #track-info {
                    slot {}
                }

                #primary-controls data-state="play" {
                    .buttons {
                        button type="button" #playpause {
                            (icon::PLAY)
                            (icon::PAUSE)
                        }
                    }

                    #progress {
                        span #time .time {
                            (TIME_PLACEHOLDER)
                        }

                        progress value="0" min="0" {}

                        span #duration .time {
                            (TIME_PLACEHOLDER)
                        }
                    }
                }

                #secondary-controls {
                    button type="button" #close { (icon::X) }
                }
            }
        }
    }
}
