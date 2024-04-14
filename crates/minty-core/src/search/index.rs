use super::ResponseExt;

use crate::{db::Id, Result};

use elasticsearch::{
    indices::{IndicesCreateParts, IndicesDeleteParts, IndicesRefreshParts},
    params::Refresh,
    BulkOperation, BulkParts, CreateParts, DeleteParts, Elasticsearch,
    SearchParts, UpdateParts,
};
use log::debug;
use minty::Uuid;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};

#[derive(Deserialize)]
struct Total {
    value: u32,
}

#[derive(Deserialize)]
struct Hit {
    #[serde(rename = "_id")]
    id: Uuid,
}

#[derive(Deserialize)]
struct Hits {
    total: Total,
    hits: Vec<Hit>,
}

#[derive(Deserialize)]
struct SearchResult {
    hits: Hits,
}

impl From<SearchResult> for minty::SearchResult<Uuid> {
    fn from(value: SearchResult) -> Self {
        Self {
            total: value.hits.total.value,
            hits: value.hits.hits.into_iter().map(|hit| hit.id).collect(),
        }
    }
}

type Config = fn() -> Json;

#[derive(Debug)]
pub struct Index {
    client: Elasticsearch,
    name: String,
    config: Config,
    refresh: Refresh,
}

impl Index {
    fn new(
        client: Elasticsearch,
        namespace: &str,
        name: &str,
        config: Config,
        refresh: Refresh,
    ) -> Self {
        Self {
            client,
            name: format!("{namespace}-{name}"),
            config,
            refresh,
        }
    }

    pub async fn bulk_create<T>(&self, items: &[T]) -> Result<()>
    where
        T: Id + Serialize,
    {
        let ops: Vec<BulkOperation<&T>> = items
            .iter()
            .map(|item| BulkOperation::create(item.id(), item).into())
            .collect();

        self.client
            .bulk(BulkParts::Index(&self.name))
            .body(ops)
            .send()
            .await?
            .check()
            .await?;

        Ok(())
    }

    pub async fn create(&self) -> Result<()> {
        self.client
            .indices()
            .create(IndicesCreateParts::Index(&self.name))
            .body((self.config)())
            .send()
            .await?
            .check()
            .await?;

        Ok(())
    }

    pub async fn delete(&self) -> Result<()> {
        self.client
            .indices()
            .delete(IndicesDeleteParts::Index(&[self.name.as_str()]))
            .ignore_unavailable(true)
            .send()
            .await?
            .check()
            .await?;

        Ok(())
    }

    pub async fn recreate(&self) -> Result<()> {
        self.delete().await?;
        self.create().await
    }

    pub async fn refresh(&self) -> Result<()> {
        self.client
            .indices()
            .refresh(IndicesRefreshParts::Index(&[&self.name]))
            .send()
            .await?
            .check()
            .await?;

        Ok(())
    }

    pub async fn search(
        &self,
        query: Json,
    ) -> Result<minty::SearchResult<Uuid>> {
        debug!("search {}: {query}", self.name);

        let result: SearchResult = self
            .client
            .search(SearchParts::Index(&[&self.name]))
            .body(query)
            .send()
            .await?
            .check()
            .await?
            .json()
            .await?;

        Ok(result.into())
    }

    pub async fn create_doc<T>(&self, id: Uuid, doc: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.client
            .create(CreateParts::IndexId(&self.name, &id.to_string()))
            .refresh(self.refresh)
            .body(doc)
            .send()
            .await?
            .check()
            .await?;

        Ok(())
    }

    pub async fn delete_doc(&self, id: Uuid) -> Result<()> {
        self.client
            .delete(DeleteParts::IndexId(&self.name, &id.to_string()))
            .refresh(self.refresh)
            .send()
            .await?
            .check()
            .await?;

        Ok(())
    }

    pub async fn update_doc(&self, id: Uuid, script: Json) -> Result<()> {
        self.client
            .update(UpdateParts::IndexId(&self.name, &id.to_string()))
            .refresh(self.refresh)
            .body(script)
            .send()
            .await?
            .check()
            .await?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Indices {
    client: Elasticsearch,
    pub post: Index,
    pub tag: Index,
}

impl Indices {
    pub fn new(
        client: Elasticsearch,
        namespace: &str,
        refresh: Refresh,
    ) -> Self {
        Self {
            client: client.clone(),
            post: Index::new(client.clone(), namespace, "post", post, refresh),
            tag: Index::new(client.clone(), namespace, "tag", tag, refresh),
        }
    }

    pub async fn create(&self) -> Result<()> {
        for index in self.all() {
            index.create().await?;
        }

        Ok(())
    }

    pub async fn delete(&self) -> Result<()> {
        let indices = self.all().map(|index| index.name.as_str());

        self.client
            .indices()
            .delete(IndicesDeleteParts::Index(&indices))
            .ignore_unavailable(true)
            .send()
            .await?
            .check()
            .await?;

        Ok(())
    }

    fn all(&self) -> [&Index; 2] {
        [&self.post, &self.tag]
    }
}

fn post() -> Json {
    json!({
        "mappings": {
            "properties": {
                "title": {
                    "type": "text",
                    "fields": {
                        "keyword": {
                            "type": "keyword"
                        }
                    }
                },
                "description": {
                    "type": "text"
                },
                "visibility": {
                    "type": "keyword"
                },
                "created": {
                    "type": "date"
                },
                "modified": {
                    "type": "date"
                },
                "tags": {
                    "type": "keyword"
                }
            }
        }
    })
}

fn tag() -> Json {
    json!({
        "mappings": {
            "properties": {
                "names": {
                    "type": "search_as_you_type"
                }
            }
        }
    })
}
