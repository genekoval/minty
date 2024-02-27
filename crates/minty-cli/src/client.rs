use crate::{
    conf::Server,
    output::{About, Output, Print},
};

use minty::{http, Repo, Uuid};

pub type Result = crate::Result<()>;

pub struct Client {
    server: String,
    repo: http::Repo,
    output: Output,
}

impl Client {
    pub fn new(alias: &str, server: &Server, output: Output) -> Self {
        Self {
            server: alias.into(),
            repo: http::Repo::new(&server.url),
            output,
        }
    }

    fn print<T: Print>(&self, t: T) -> Result {
        Ok(t.print(self.output)?)
    }

    pub async fn about(&self) -> Result {
        let info = self.repo.about().await?;
        let about = About {
            server: &self.server,
            url: self.repo.url(),
            info,
        };

        self.print(about)
    }

    pub async fn get_tag(&self, id: Uuid) -> Result {
        self.print(self.repo.get_tag(id).await?)
    }
}
