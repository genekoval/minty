use log::error;
use num_format::{SystemLocale, ToFormattedString};
use std::sync::OnceLock;

pub trait FormatNumber {
    fn format(self) -> String;
}

impl FormatNumber for u64 {
    fn format(self) -> String {
        match locale() {
            Some(locale) => self.to_formatted_string(locale),
            None => self.to_string(),
        }
    }
}

fn locale() -> Option<&'static SystemLocale> {
    static INSTANCE: OnceLock<Option<SystemLocale>> = OnceLock::new();

    INSTANCE
        .get_or_init(|| {
            SystemLocale::default()
                .inspect_err(|err| {
                    error!("Failed to load system locale: {err}")
                })
                .ok()
        })
        .as_ref()
}
