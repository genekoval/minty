use minty::{
    http::{self, cookie::Jar, Credentials},
    Login, Repo, SignUp, Url,
};
use std::{
    env::{self, VarError},
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, OnceLock,
    },
};
use timber::Sink;
use tokio::sync::OnceCell;

const LOG_VAR: &str = "MINTY_TEST_LOG";
const URL_VAR: &str = "MINTY_TEST_URL";

pub fn repo() -> http::Repo {
    http::Repo::build(url())
        .credentials(Credentials::Cookies)
        .build()
        .unwrap()
}

pub async fn admin() -> http::Repo {
    static CREDENTIALS: OnceCell<Credentials> = OnceCell::const_new();

    let url = url();
    let credentials = CREDENTIALS
        .get_or_init(|| async { authenticate(&url).await })
        .await
        .clone();

    http::Repo::build(url)
        .credentials(credentials)
        .build()
        .unwrap()
}

pub fn sign_up_info(name: &str) -> SignUp {
    let email = format!("{name}@example.com");
    let password = format!("{name} password");

    SignUp {
        username: name.parse().unwrap(),
        email: email.parse().unwrap(),
        password: password.parse().unwrap(),
    }
}

pub async fn sign_up(info: &SignUp) -> http::Repo {
    let repo = repo();

    repo.sign_up(info, None).await.unwrap();

    repo
}

pub async fn new_user(name: &str) -> http::Repo {
    let info = sign_up_info(name);
    sign_up(&info).await
}

pub async fn next_user() -> http::Repo {
    static COUNTER: OnceLock<AtomicU64> = OnceLock::new();

    let counter = COUNTER.get_or_init(|| AtomicU64::new(0));
    let next = counter.fetch_add(1, Ordering::Relaxed);
    let name = format!("minty{next}");

    new_user(&name).await
}

async fn authenticate(url: &Url) -> Credentials {
    let credentials = Credentials::CookieJar(Arc::new(Jar::default()));

    let login = Login {
        email: "minty@example.com".into(),
        password: "password".into(),
    };

    http::Repo::build(url.clone())
        .credentials(credentials.clone())
        .build()
        .unwrap()
        .authenticate(&login)
        .await
        .unwrap();

    credentials
}

fn enable_logging() {
    if let Some(log) = env::var_os(LOG_VAR) {
        let log = PathBuf::from(&log);
        timber::new().sink(Sink::File(log)).init().unwrap();
    }
}

#[must_use]
fn get_url() -> Url {
    enable_logging();

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

fn url() -> Url {
    static URL: OnceLock<Url> = OnceLock::new();
    URL.get_or_init(get_url).clone()
}
