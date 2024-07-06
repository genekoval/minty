pub use reqwest::cookie::Jar;

use crate::{Error, Result};

use bytes::Bytes;
use cookie_store::{CookieStore, RawCookie, RawCookieParseError};
use log::{debug, error};
use reqwest::header::HeaderValue;
use std::{
    fs::File,
    io::{BufReader, BufWriter, ErrorKind::NotFound},
    path::{Path, PathBuf},
    sync::RwLock,
};
use url::Url;

#[derive(Debug)]
struct CookieFileInner {
    store: CookieStore,
    path: PathBuf,
    is_updated: bool,
}

impl CookieFileInner {
    fn new(path: PathBuf) -> Result<Self> {
        let store = load_cookies(&path)?;

        Ok(Self {
            store,
            path,
            is_updated: false,
        })
    }

    fn cookies(&self, url: &Url) -> Option<HeaderValue> {
        let string = self
            .store
            .get_request_values(url)
            .map(|(name, value)| format!("{}={}", name, value))
            .collect::<Vec<_>>()
            .join("; ");

        if string.is_empty() {
            None
        } else {
            HeaderValue::from_maybe_shared(Bytes::from(string)).ok()
        }
    }

    fn set_cookies(
        &mut self,
        cookie_headers: &mut dyn Iterator<Item = &HeaderValue>,
        url: &Url,
    ) {
        let cookies = cookie_headers.filter_map(|value| {
            std::str::from_utf8(value.as_bytes())
                .map_err(RawCookieParseError::from)
                .and_then(RawCookie::parse)
                .map(|cookie| cookie.into_owned())
                .ok()
        });

        self.store.store_response_cookies(cookies, url);
        self.is_updated = true;
    }

    fn write_cookies(&mut self) -> Result<()> {
        if self.is_updated {
            let path = self.path.as_path();

            let mut file =
                File::create(path).map(BufWriter::new).map_err(|err| {
                    Error::other(format!(
                        "failed to open cookie file '{}': {err}",
                        path.display()
                    ))
                })?;

            self.store.save_json(&mut file).map_err(|err| {
                Error::other(format!(
                    "failed to write cookies to file '{}': {err}",
                    path.display()
                ))
            })?;

            debug!("saved cookies to '{}'", path.display());

            self.is_updated = false;
        }

        Ok(())
    }
}

impl Drop for CookieFileInner {
    fn drop(&mut self) {
        if let Err(err) = self.write_cookies() {
            error!("{err}");
        }
    }
}

#[derive(Debug)]
pub struct CookieFile(RwLock<CookieFileInner>);

impl CookieFile {
    pub fn new(path: PathBuf) -> Result<Self> {
        Ok(Self(RwLock::new(CookieFileInner::new(path)?)))
    }
}

impl reqwest::cookie::CookieStore for CookieFile {
    fn set_cookies(
        &self,
        cookie_headers: &mut dyn Iterator<Item = &HeaderValue>,
        url: &Url,
    ) {
        self.0.write().unwrap().set_cookies(cookie_headers, url);
    }

    fn cookies(&self, url: &url::Url) -> Option<HeaderValue> {
        self.0.read().unwrap().cookies(url)
    }
}

fn load_cookies(path: &Path) -> Result<CookieStore> {
    let store = match File::open(path).map(BufReader::new) {
        Ok(file) => CookieStore::load_json(file).map_err(|err| {
            Error::other(format!(
                "failed to load cookies from file '{}': {err}",
                path.display()
            ))
        })?,
        Err(err) => match err.kind() {
            NotFound => CookieStore::new(None),
            _ => {
                return Err(Error::other(format!(
                    "failed to open cookie file '{}': {err}",
                    path.display()
                )))
            }
        },
    };

    debug!("loaded cookies from '{}'", path.display());

    Ok(store)
}
