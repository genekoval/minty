use super::{icon, Html};

use maud::{html, Markup};
use minty::http::ObjectExt;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Object(pub minty::Object);

impl Html for Object {
    fn page_title(&self) -> &str {
        "Object"
    }

    fn full(&self) -> Markup {
        html! {
            #modal
                _="on closeModal add .closing then \
                    wait for animationend then remove me"
            {
                .modal-underlay _="on click trigger closeModal" {}
                .modal-content
                    _="on keydown[key is 'Escape'] from document \
                        trigger closeModal"
                {
                    #image-viewer {
                        img .modal-zoom .displayed src=(self.0.data_path());

                        button
                            .close-modal
                            _="on click trigger closeModal"
                        {
                            (icon::X)
                        }
                    }
                }
            }
        }
    }
}
