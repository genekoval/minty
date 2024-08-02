use super::Icon;

use ago::{Relative, Unit::Second};
use maud::{html, Render};

const DATE_FORMAT: &str = "%A, %B %-d, %Y at %r";

#[derive(Clone, Copy, Debug)]
pub struct DateTime {
    value: minty::DateTime,
    icon: Option<Icon>,
    prefix: Option<&'static str>,
    font_smaller: bool,
    secondary: bool,
}

impl DateTime {
    pub fn new(value: minty::DateTime) -> Self {
        Self {
            value,
            icon: None,
            prefix: None,
            font_smaller: false,
            secondary: false,
        }
    }

    pub fn font_smaller(mut self) -> Self {
        self.font_smaller = true;
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn prefix(mut self, prefix: &'static str) -> Self {
        self.prefix = Some(prefix);
        self
    }

    pub fn secondary(mut self) -> Self {
        self.secondary = true;
        self
    }

    fn long_format(&self) -> String {
        let formatted = self.value.format(DATE_FORMAT);
        let relative = self
            .value
            .relative()
            .granularity(Second)
            .with_tense(true)
            .long_format();

        format!("{relative} on {formatted}")
    }
}

impl Render for DateTime {
    fn render(&self) -> maud::Markup {
        let mut date_time = self.long_format();

        if let Some(prefix) = self.prefix {
            date_time = format!("{prefix} {date_time}")
        }

        html! {
            span .font-smaller[self.font_smaller] .secondary[self.secondary] {
                @if let Some(icon) = self.icon {
                    (icon)
                    span ."label-text" { (date_time) }
                } @else {
                    (date_time)
                }
            }
        }
    }
}
