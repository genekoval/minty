use ago::{Relative, Unit::Second};
use maud::{html, Render};

const DATE_FORMAT: &str = "%A, %B %-d, %Y at %r";

#[derive(Clone, Copy, Debug)]
pub struct DateTime {
    value: minty::DateTime,
    prefix: Option<&'static str>,
}

impl DateTime {
    pub fn new(value: minty::DateTime) -> Self {
        Self {
            value,
            prefix: None,
        }
    }

    pub fn prefix(mut self, prefix: &'static str) -> Self {
        self.prefix = Some(prefix);
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

impl From<minty::DateTime> for DateTime {
    fn from(value: minty::DateTime) -> Self {
        Self::new(value)
    }
}

impl Render for DateTime {
    fn render(&self) -> maud::Markup {
        let mut date_time = self.long_format();

        if let Some(prefix) = self.prefix {
            date_time = format!("{prefix} {date_time}")
        }

        html! {
            span .secondary .font-smaller { (date_time) }
        }
    }
}
