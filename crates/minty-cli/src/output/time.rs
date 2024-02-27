use chrono::{Duration, Local};
use minty::DateTime;

const DATE_FORMAT: &str = "%A, %B %-d, %Y at %r";

const DAYS_IN_MONTH: i64 = 30;
const DAYS_IN_YEAR: i64 = 365;

const NOW: &str = "just now";

const UNITS: [&str; 7] =
    ["year", "month", "week", "day", "hour", "minute", "second"];

pub trait FormatDate {
    fn long_date(&self) -> String;
}

impl FormatDate for DateTime {
    fn long_date(&self) -> String {
        let relative = relative_full(*self);
        let formatted = self.format(DATE_FORMAT);

        format!("{relative} on {formatted}")
    }
}

fn relative_full(date_time: DateTime) -> String {
    let duration = Local::now() - date_time;

    if duration.is_zero() {
        return NOW.into();
    }

    let past = duration > Duration::zero();
    let duration = duration.abs();

    let years = duration.num_days() / DAYS_IN_YEAR;
    let duration = duration - Duration::days(DAYS_IN_YEAR * years);

    let months = duration.num_days() / DAYS_IN_MONTH;
    let duration = duration - Duration::days(DAYS_IN_MONTH * months);

    let weeks = duration.num_weeks();
    let duration = duration - Duration::weeks(weeks);

    let days = duration.num_days();
    let duration = duration - Duration::days(days);

    let hours = duration.num_hours();
    let duration = duration - Duration::hours(hours);

    let minutes = duration.num_minutes();
    let duration = duration - Duration::minutes(minutes);

    let seconds = duration.num_seconds();

    let durations = [years, months, weeks, days, hours, minutes, seconds];

    let Some(position) = durations.iter().position(|&duration| duration > 0)
    else {
        return NOW.into();
    };

    let count = *durations.get(position).unwrap();
    let mut duration = duration_with_unit(count, position);

    let position = position + 1;

    if let Some(count) = durations.get(position) {
        if *count != 0 {
            let next = duration_with_unit(*count, position);
            duration = format!("{duration}, {next}");
        }
    }

    if past {
        format!("{duration} ago")
    } else {
        format!("in {duration}")
    }
}

fn duration_with_unit(count: i64, unit: usize) -> String {
    let unit = *UNITS
        .get(unit)
        .expect("durations and units arrays should be the same size");

    format!(
        "{count} {unit}{}",
        match count {
            1 => "",
            _ => "s",
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative() {
        let now = Local::now();

        assert_eq!(relative_full(now), NOW);

        let delta = Duration::days(1) + Duration::hours(12);

        assert_eq!(relative_full(now - delta), "1 day, 12 hours ago");
        assert_eq!(relative_full(now + delta), "in 1 day, 11 hours");
    }
}
