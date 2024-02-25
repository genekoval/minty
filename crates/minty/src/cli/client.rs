use crate::cli::{
    self,
    conf::Server,
    output::{About, Output, Print},
};

use minty::{http, Repo};

pub type Result = cli::Result<()>;

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

    fn print<T: Print>(&self, t: T) {
        t.print(self.output)
    }

    pub async fn about(&self) -> Result {
        let info = self.repo.about().await?;
        let about = About {
            server: &self.server,
            url: self.repo.url(),
            info,
        };

        self.print(about);
        Ok(())
    }
}
