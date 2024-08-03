use super::{Icon, Space};

use ago::{Relative, RelativeBuilder, Unit::Second};
use maud::{html, Render};

const DATE_FORMAT: &str = "%A, %B %-d, %Y at %r";

#[derive(Clone, Copy, Debug)]
pub struct DateTime {
    value: minty::DateTime,
    icon: Option<Icon>,
    prefix: Option<&'static str>,
}

impl DateTime {
    pub fn new(value: minty::DateTime) -> Self {
        Self {
            value,
            icon: None,
            prefix: None,
        }
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn prefix(mut self, prefix: &'static str) -> Self {
        self.prefix = Some(prefix);
        self
    }
    fn relative(&self) -> RelativeBuilder {
        self.value.relative().granularity(Second).with_tense(true)
    }
}

impl Render for DateTime {
    fn render(&self) -> maud::Markup {
        let relative = self.relative();

        html! {
            span {
                @if let Some(icon) = self.icon {
                    (icon.inline())
                }

                span .label-text[self.icon.is_some()] {
                    @if let Some(prefix) = self.prefix {
                        (prefix)
                        (Space)
                    }

                    span .bold { (relative.long_format()) }

                    (Space)
                    "on"
                    (Space)

                    (self.value.format(DATE_FORMAT))
                }
            }
        }
    }
}
