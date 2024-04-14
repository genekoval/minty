use crate::{DateTime, Error, ErrorKind, ObjectSummary, Result, Url, Uuid};

use bytes::Bytes;
use futures_core::{Stream, TryStream};
use log::debug;
use mime::{Mime, TEXT_PLAIN_UTF_8};
use reqwest::{header::CONTENT_TYPE, Body, Method, Request, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use std::{error, io};
use tokio_stream::StreamExt;

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

pub struct Response {
    inner: reqwest::Response,
}

impl Response {
    pub async fn deserialize<T: DeserializeOwned>(self) -> Result<T> {
        self.inner.json().await.map_read_err()
    }

    pub async fn text(self) -> Result<String> {
        self.inner.text().await.map_read_err()
    }

    pub async fn date_time(self) -> Result<DateTime> {
        let text = self.text().await?;
        text.parse().map_err(|err| {
            Error::other(format!(
                "received invalid date from server '{text}': {err}"
            ))
        })
    }

    pub async fn uuid(self) -> Result<Uuid> {
        let text = self.text().await?;
        Uuid::try_parse(&text).map_err(|err| {
            Error::other(format!(
                "received invalid UUID from server '{text}': {err}"
            ))
        })
    }

    pub fn object(
        self,
    ) -> Result<(ObjectSummary, impl Stream<Item = io::Result<Bytes>>)> {
        let content_length: u64 = self
            .inner
            .headers()
            .get("content-length")
            .expect("content-length header should be present")
            .to_str()
            .ok()
            .and_then(|value| value.parse().ok())
            .expect("content-length header should be a valid number");

        let content_type: String = self
            .inner
            .headers()
            .get("content-type")
            .expect("content-type header should be present")
            .to_str()
            .expect("content-type header should be valid ASCII")
            .into();

        let summary = ObjectSummary {
            media_type: content_type,
            size: content_length,
        };

        let stream = self
            .inner
            .bytes_stream()
            .map(|result| result.map_err(io::Error::other));

        Ok((summary, stream))
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

    pub fn text(mut self, body: String) -> Self {
        self.inner = self.inner.body(body);
        self.content_type(TEXT_PLAIN_UTF_8)
    }

    pub fn serialize<T>(mut self, body: &T) -> Self
    where
        T: Serialize + ?Sized,
    {
        self.inner = self.inner.json(body);
        self
    }

    pub fn stream<S>(mut self, stream: S) -> Self
    where
        S: TryStream + Send + Sync + 'static,
        S::Error: Into<Box<dyn error::Error + Send + Sync>>,
        Bytes: From<S::Ok>,
    {
        self.inner = self.inner.body(Body::wrap_stream(stream));
        self
    }

    pub async fn send(self) -> Result<Response> {
        let (client, request) = self.inner.build_split();
        let request = request.map_err(|err| {
            Error::other(format!("failed to build request: {err}"))
        })?;

        let method = request.method().clone();

        let response = client
            .execute(request)
            .await
            .map_err(|err| Error::other(format!("request failed: {err}")))?;

        let status = response.status();
        let url = response.url().clone().to_string();

        debug!("({status}) {method} {url}");

        if status.is_success() {
            return Ok(Response { inner: response });
        }

        let kind = if status == StatusCode::NOT_FOUND {
            ErrorKind::NotFound
        } else if status.is_client_error() {
            ErrorKind::Client
        } else if status.is_server_error() {
            ErrorKind::Server
        } else {
            ErrorKind::Other
        };

        let mut message = response.text().await.map_read_err()?;

        if message.is_empty() {
            message = format!("{method} {url}: {status}");
        }

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
