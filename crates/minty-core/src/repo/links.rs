use super::Repo;

use crate::{Error, Result};

use minty::{Source, Url};

pub struct Links<'a> {
    repo: &'a Repo,
}

impl<'a> Links<'a> {
    pub fn new(repo: &'a Repo) -> Self {
        Self { repo }
    }

    pub async fn add(&self, url: &Url) -> Result<Source> {
        const HOST_PREFIX: &str = "www.";

        let Some(host) = url.host_str() else {
            return Err(Error::InvalidInput(
                "expected a host in the source URL".into(),
            ));
        };

        let host = host.strip_prefix(HOST_PREFIX).unwrap_or(host);
        let scheme = url.scheme();

        let site = match self.repo.database.read_site(scheme, host).await? {
            (Some(site),) => site,
            (None,) => self.add_site(scheme, host).await?,
        };

        let mut resource = String::from(url.path());

        if let Some(query) = url.query() {
            if !query.is_empty() {
                resource = format!("{resource}?{query}");
            }
        }

        if let Some(fragment) = url.fragment() {
            if !fragment.is_empty() {
                resource = format!("{resource}#{fragment}");
            }
        }

        Ok(self
            .repo
            .database
            .create_source(site, &resource)
            .await?
            .into())
    }

    async fn add_site(&self, scheme: &str, host: &str) -> Result<i64> {
        let icon = self.repo.favicons.download_icon(scheme, host).await;
        let site = self.repo.database.create_site(scheme, host, icon).await?;

        Ok(site.site_id)
    }
}
