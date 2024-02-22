use crate::{Error, ErrorKind, Result, Url};

use mime::{Mime, TEXT_PLAIN_UTF_8};
use reqwest::{header::CONTENT_TYPE, Method, Request};
use serde::{de::DeserializeOwned, Serialize};

trait MapReadErr {
    type Output;

    fn map_read_err(self) -> Result<Self::Output>;
}

impl<T> MapReadErr for reqwest::Result<T> {
    type Output = T;

    fn map_read_err(self) -> Result<Self::Output> {
        self.map_err(|err| {
            Error::other(format!("failed to read response body: {err}"))
        })
    }
}

#[derive(Debug)]
pub struct RequestBuilder {
    inner: reqwest::RequestBuilder,
}

impl RequestBuilder {
    pub fn from_parts(client: reqwest::Client, request: Request) -> Self {
        Self {
            inner: reqwest::RequestBuilder::from_parts(client, request),
        }
    }

    pub fn query<T>(mut self, query: &T) -> Self
    where
        T: Serialize,
    {
        self.inner = self.inner.query(query);
        self
    }

    fn content_type(mut self, mime: Mime) -> Self {
        self.inner = self.inner.header(CONTENT_TYPE, mime.as_ref());
        self
    }

    pub fn text(mut self, body: &str) -> Self {
        self.inner = self.inner.body(String::from(body));
        self.content_type(TEXT_PLAIN_UTF_8)
    }

    pub fn serialize<T>(mut self, body: &T) -> Self
    where
        T: Serialize + ?Sized,
    {
        self.inner = self.inner.json(body);
        self
    }

    pub async fn send<T: DeserializeOwned>(self) -> Result<T> {
        let response =
            self.inner.send().await.map_err(|err| {
                Error::other(format!("request failed: {err}"))
            })?;

        let status = response.status();

        if status.is_success() {
            let body = response.json().await.map_read_err()?;
            return Ok(body);
        }

        let kind = if status.is_client_error() {
            ErrorKind::Client
        } else if status.is_server_error() {
            ErrorKind::Server
        } else {
            ErrorKind::Other
        };

        let message = response.text().await.map_read_err()?;

        Err(Error::new(kind, message))
    }
}

#[derive(Clone, Debug)]
pub struct Client {
    url: Url,
    client: reqwest::Client,
}

impl Client {
    pub fn new(url: &Url) -> Self {
        Self {
            url: url.clone(),
            client: reqwest::Client::new(),
        }
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn delete<P>(&self, path: P) -> RequestBuilder
    where
        P: AsRef<str>,
    {
        self.request(Method::DELETE, path)
    }

    pub fn get<P>(&self, path: P) -> RequestBuilder
    where
        P: AsRef<str>,
    {
        self.request(Method::GET, path)
    }

    pub fn post<P>(&self, path: P) -> RequestBuilder
    where
        P: AsRef<str>,
    {
        self.request(Method::POST, path)
    }

    pub fn put<P>(&self, path: P) -> RequestBuilder
    where
        P: AsRef<str>,
    {
        self.request(Method::PUT, path)
    }

    fn request<P>(&self, method: Method, path: P) -> RequestBuilder
    where
        P: AsRef<str>,
    {
        let path = path.as_ref();
        let mut url = self.url.clone();

        if !path.is_empty() {
            url.set_path(path);
        }

        let request = Request::new(method, url);

        RequestBuilder::from_parts(self.client.clone(), request)
    }
}
