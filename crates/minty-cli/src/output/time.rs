use chrono::{Duration, Local};
use minty::DateTime;
use std::{
    cmp::Ordering,
    fmt::{self, Display},
};

const DATE_FORMAT: &str = "%A, %B %-d, %Y at %r";

const SECONDS_PER_MINUTE: i64 = 60;
const SECONDS_PER_HOUR: i64 = SECONDS_PER_MINUTE * 60;
const SECONDS_PER_DAY: i64 = SECONDS_PER_HOUR * 24;
const SECONDS_PER_WEEK: i64 = SECONDS_PER_DAY * 7;
const SECONDS_PER_MONTH: i64 = SECONDS_PER_DAY * 30;
const SECONDS_PER_YEAR: i64 = SECONDS_PER_DAY * 365;

pub trait FormatDate {
    fn long_date(self) -> String;
}

impl FormatDate for DateTime {
    fn long_date(self) -> String {
        let relative = self.relative_long(2);
        let formatted = self.format(DATE_FORMAT);

        format!("{relative} on {formatted}")
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(i64)]
pub enum Unit {
    Year = SECONDS_PER_YEAR,
    Month = SECONDS_PER_MONTH,
    Week = SECONDS_PER_WEEK,
    Day = SECONDS_PER_DAY,
    Hour = SECONDS_PER_HOUR,
    Minute = SECONDS_PER_MINUTE,
    Second = 1,
}

impl Unit {
    pub fn full(&self) -> &'static str {
        use Unit::*;

        match self {
            Year => "year",
            Month => "month",
            Week => "week",
            Day => "day",
            Hour => "hour",
            Minute => "minute",
            Second => "second",
        }
    }

    pub fn abbrev(&self) -> &'static str {
        use Unit::*;

        match self {
            Year => "y",
            Month => "mo",
            Week => "w",
            Day => "d",
            Hour => "h",
            Minute => "mi",
            Second => "s",
        }
    }
}

impl Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.full())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Tense {
    Past,
    Present,
    Future,
}

#[derive(Clone, Copy, Debug)]
pub struct DurationPart {
    count: i64,
    unit: Unit,
}

impl Display for DurationPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}{}",
            self.count,
            self.unit,
            match self.count {
                1 => "",
                _ => "s",
            }
        )
    }
}

pub struct Relative<U> {
    duration: Duration,
    unit: U,
    tense: Tense,
}

impl<U> Relative<U> {
    fn new(duration: Duration, unit: U) -> Self {
        let tense = match duration.cmp(&Duration::zero()) {
            Ordering::Less => Tense::Future,
            Ordering::Equal => Tense::Present,
            Ordering::Greater => Tense::Past,
        };

        Self {
            duration: duration.abs(),
            unit,
            tense,
        }
    }
}

impl<U> Iterator for Relative<U>
where
    U: Iterator<Item = Unit>,
{
    type Item = DurationPart;

    fn next(&mut self) -> Option<Self::Item> {
        let unit = self.unit.next()?;
        let seconds_per_unit = unit as i64;
        let count = self.duration.num_seconds() / seconds_per_unit;

        self.duration -= Duration::seconds(count * seconds_per_unit);

        Some(DurationPart { count, unit })
    }
}

pub trait RelativeUnits: Sized {
    fn relative_units(self) -> Relative<impl Iterator<Item = Unit>>;

    fn relative_long(self, units: usize) -> String {
        let relative = self.relative_units();
        let tense = relative.tense;

        let duration = relative
            .filter(|duration| duration.count > 0)
            .take(units)
            .map(|duration| duration.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        match tense {
            Tense::Past => format!("{duration} ago"),
            Tense::Present => "just now".into(),
            Tense::Future => format!("in {duration}"),
        }
    }

    fn relative_abbrev(self, units: usize) -> String {
        let relative = self.relative_units();
        let tense = relative.tense;

        let duration = relative
            .filter(|duration| duration.count > 0)
            .take(units)
            .map(|duration| {
                format!("{}{}", duration.count, duration.unit.abbrev())
            })
            .collect::<Vec<_>>()
            .join(", ");

        match tense {
            Tense::Past => format!("{duration} ago"),
            Tense::Present => "now".into(),
            Tense::Future => format!("in {duration}"),
        }
    }
}

impl RelativeUnits for Duration {
    fn relative_units(self) -> Relative<impl Iterator<Item = Unit>> {
        use Unit::*;

        let units = [Year, Month, Week, Day, Hour, Minute, Second];
        Relative::new(self, units.into_iter())
    }
}

impl RelativeUnits for DateTime {
    fn relative_units(self) -> Relative<impl Iterator<Item = Unit>> {
        let duration = Local::now() - self;
        duration.relative_units()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative() {
        let now = Local::now();
        let delta = Duration::days(1) + Duration::hours(12);

        assert_eq!((now - delta).relative_long(2), "1 day, 12 hours ago");
        assert_eq!((now + delta).relative_long(2), "in 1 day, 11 hours");
    }
}
