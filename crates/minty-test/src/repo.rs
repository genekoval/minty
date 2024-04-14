use minty::{http, Repo, Url};
use std::{
    env::{self, VarError},
    path::PathBuf,
    sync::OnceLock,
};
use timber::Sink;

const LOG_VAR: &str = "MINTY_TEST_LOG";
const URL_VAR: &str = "MINTY_TEST_URL";

pub fn repo() -> http::Repo {
    static URL: OnceLock<Url> = OnceLock::new();

    let url = URL.get_or_init(|| {
        enable_logging();
        get_url()
    });

    http::Repo::new(url)
}

fn enable_logging() {
    if let Some(log) = env::var_os(LOG_VAR) {
        let log = PathBuf::from(&log);
        timber::new().sink(Sink::File(log)).init().unwrap();
    }
}

#[must_use]
fn get_url() -> Url {
    let env = env::var(URL_VAR).unwrap_or_else(|err| match err {
        VarError::NotPresent => {
            panic!("environment variable '{URL_VAR}' not set")
        }
        VarError::NotUnicode(_) => {
            panic!("{URL_VAR} not set to a unicode string")
        }
    });

    Url::parse(&env)
        .unwrap_or_else(|err| panic!("failed to parse {URL_VAR}: {err}"))
}
