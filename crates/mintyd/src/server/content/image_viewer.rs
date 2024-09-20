use super::icon;

use maud::{html, Markup, Render};
use minty::{http::ObjectExt, ObjectPreview};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ImageViewer<'a> {
    images: &'a [ObjectPreview],
    index: usize,
}

impl<'a> ImageViewer<'a> {
    pub fn new(images: &'a [ObjectPreview], index: usize) -> Self {
        Self { images, index }
    }
}

impl<'a> Render for ImageViewer<'a> {
    fn render(&self) -> Markup {
        let last = self.images.len() - 1;

        html! {
            #modal
                _="on closeModal add .closing then \
                    wait for animationend then remove me"
            {
                .modal-underlay _="on click trigger closeModal" {}
                #image-viewer .modal-content
                    _="on toggleControls debounced at 200ms toggle .hide-controls end \
                        on keydown[key is 'Escape'] from document \
                        halt the event trigger closeModal"
                {
                    @for (index, image) in self.images.iter().enumerate() {
                        (Slide {
                            image,
                            index,
                            last,
                            is_active: index == self.index
                        })
                    }

                    button
                        .control
                        .close-modal
                        _="on click trigger closeModal"
                    {
                        (icon::X)
                    }

                    button
                        #previous-image
                        .control
                        .previous
                        _="on click \
                            decrement the @data-index of #image-viewer \
                            then go to url `#image-${result}`"
                    {
                        (icon::CHEVRON_LEFT)
                    }

                    button
                        #next-image
                        .control
                        .next
                        _="on click \
                            increment the @data-index of #image-viewer \
                            then go to url `#image-${result}`"
                    {
                        (icon::CHEVRON_RIGHT)
                    }
                }
            }
        }
    }
}

struct Slide<'a> {
    image: &'a ObjectPreview,
    index: usize,
    last: usize,
    is_active: bool,
}

impl<'a> Render for Slide<'a> {
    fn render(&self) -> Markup {
        let index = self.index;
        let id = format!("image-{index}");

        let mut script = format!(
            "on dblclick halt the event log 'Double clicked!' \
                on click halt the event trigger toggleControls end \
                on intersection(intersecting) if intersecting \
                set #image-viewer@data-index to '{index}' then "
        );

        if self.index == 0 {
            script.push_str("hide #previous-image");
        } else {
            script.push_str("show #previous-image");
        }

        script.push_str(" then ");

        if self.index == self.last {
            script.push_str("hide #next-image");
        } else {
            script.push_str("show #next-image");
        }

        if !self.is_active {
            script.push_str(
                " then if no @src \
                set @src to @data-src then remove @data-src end",
            );
        } else {
            script.push_str(&format!(
                " end on load go to #{id} then \
                    add .smooth-scroll to #image-viewer"
            ));
        }

        let path = self.image.data_path();
        let path = path.as_str();

        let src = self.is_active.then_some(path);
        let data_src = (!self.is_active).then_some(path);

        html! {
            #(id) .slide _="on click trigger closeModal" {
                img
                    .modal-zoom
                    _=(script)
                    data-index=(self.index)
                    data-src=[data_src]
                    src=[src];
            }
        }
    }
}
