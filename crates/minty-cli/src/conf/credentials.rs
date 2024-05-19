use minty::Url;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

type Servers = BTreeMap<Url, Users>;
type Users = BTreeMap<String, String>;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Credentials(Servers);

impl Credentials {
    pub fn get(&self, server: &Url, email: &str) -> Option<String> {
        self.0
            .get(server)
            .and_then(|users| users.get(email).cloned())
    }

    pub fn insert(&mut self, server: Url, email: String, secret: String) {
        if let Some(users) = self.0.get_mut(&server) {
            users.insert(email, secret);
        } else {
            let mut users = Users::new();
            users.insert(email, secret);
            self.0.insert(server, users);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn remove(&mut self, server: &Url, email: &str) {
        if let Some(users) = self.0.get_mut(server) {
            users.remove(email);

            if users.is_empty() {
                self.0.remove(server);
            }
        }
    }
}
