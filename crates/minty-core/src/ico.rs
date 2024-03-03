use crate::obj::Bucket;

use log::error;
use minty::Uuid;
use reqwest::Client;
use scraper::{Html, Selector};

pub struct Favicons {
    client: Client,
    bucket: Bucket,
    selectors: Vec<Selector>,
}

impl Favicons {
    pub fn new(bucket: Bucket) -> Self {
        Self {
            client: Client::new(),
            bucket,
            selectors: vec![
                r#"link[rel="icon"]"#,
                r#"link[rel="shortcut icon"]"#,
            ]
            .into_iter()
            .map(|selector| Selector::parse(selector).unwrap())
            .collect(),
        }
    }

    pub async fn download_icon(
        &self,
        scheme: &str,
        host: &str,
    ) -> Option<Uuid> {
        let root = format!("{scheme}://{host}");
        let favicon = self.get_favicon_url(&root).await;
        let stream = self
            .client
            .get(&favicon)
            .send()
            .await
            .inspect_err(|err| error!("request to {favicon} failed: {err}"))
            .ok()?
            .error_for_status()
            .inspect_err(|err| error!("request to {favicon} failed: {err}"))
            .ok()?
            .bytes_stream();

        let object = self
            .bucket
            .add_object_stream(stream)
            .await
            .inspect_err(|err| {
                error!("failed to upload favicon for {scheme} to bucket: {err}")
            })
            .ok()?;

        Some(object.id)
    }

    async fn check_html(&self, url: &str) -> Option<String> {
        let text = self
            .client
            .get(url)
            .send()
            .await
            .inspect_err(|err| error!("request to {url} failed: {err}"))
            .ok()?
            .text()
            .await
            .inspect_err(|err| {
                error!("failed to download HTML from {url}: {err}")
            })
            .ok()?;

        let html = Html::parse_document(&text);

        for selector in &self.selectors {
            let mut largest: usize = 0;
            let mut result: Option<&str> = None;

            for element in html.select(selector) {
                let Some(href) = element.attr("href") else {
                    continue;
                };

                let Some(size) = element.attr("sizes") else {
                    if result.is_none() {
                        result = Some(href);
                    }

                    continue;
                };

                let Some(size) = size.split('x').next()?.parse::<usize>().ok()
                else {
                    continue;
                };

                if size > largest {
                    largest = size;
                    result = Some(href);
                }
            }

            if let Some(href) = result {
                return Some(href.into());
            }
        }

        None
    }

    async fn get_favicon_url(&self, root: &str) -> String {
        self.check_html(root)
            .await
            .map(|favicon| {
                if !favicon.starts_with("//") && favicon.starts_with('/') {
                    format!("{root}{favicon}")
                } else {
                    favicon
                }
            })
            .unwrap_or_else(|| format!("{root}/favicon.ico"))
    }
}
