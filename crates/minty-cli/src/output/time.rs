use ago::{Relative, Unit::Second};
use minty::DateTime;

const DATE_FORMAT: &str = "%A, %B %-d, %Y at %r";

pub trait FormatDate {
    fn long_date(self) -> String;

    fn relative_abbrev(self) -> String;
}

impl FormatDate for DateTime {
    fn long_date(self) -> String {
        let formatted = self.format(DATE_FORMAT);
        let relative = self
            .relative()
            .units(2)
            .granularity(Second)
            .with_tense(true)
            .long_format();

        format!("{relative} on {formatted}")
    }

    fn relative_abbrev(self) -> String {
        self.relative().granularity(Second).abbrev()
    }
}
