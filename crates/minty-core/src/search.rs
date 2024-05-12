mod index;
mod response;

pub use index::{Index, Indices};

use response::ResponseExt;

use crate::{conf::SearchConfig, db::PostSearch, Result};

use elasticsearch::{
    http::transport::{SingleNodeConnectionPool, TransportBuilder},
    Elasticsearch,
};
use minty::{
    DateTime, PostQuery, ProfileQuery, SearchResult, Uuid, Visibility,
};
use serde_json::{json, Map, Value as Json};
use std::result;

#[derive(Debug)]
pub struct Search {
    pub indices: Indices,
}

impl Search {
    pub fn new(config: &SearchConfig) -> result::Result<Self, String> {
        let pool = SingleNodeConnectionPool::new(config.node.clone());

        let transport = TransportBuilder::new(pool)
            .auth(config.auth.clone().into())
            .build()
            .map_err(|err| {
                format!(
                    "failed to initialize Elasticsearch HTTP transport: {err}"
                )
            })?;

        let client = Elasticsearch::new(transport);
        let indices = Indices::new(client, &config.namespace, config.refresh);

        Ok(Self { indices })
    }

    pub async fn create_indices(&self) -> Result<()> {
        self.indices.create().await
    }

    pub async fn delete_indices(&self) -> Result<()> {
        self.indices.delete().await
    }

    pub async fn add_entity_alias(
        &self,
        index: &Index,
        id: Uuid,
        alias: &str,
    ) -> Result<()> {
        let script = "if (!ctx._source.names.contains(params.alias)) {\
                          ctx._source.names.add(params.alias);\
                      }";

        index
            .update_doc(
                id,
                json!({
                    "script": {
                        "lang": "painless",
                        "params": { "alias": alias },
                        "source": script,
                    },
                    "upsert": { "names": [alias] }
                }),
            )
            .await
    }

    pub async fn add_post(&self, post: &PostSearch) -> Result<()> {
        self.indices.post.create_doc(post.id, post).await
    }

    pub async fn add_post_tag(&self, post: Uuid, tag: Uuid) -> Result<()> {
        let script = "if (!ctx._source.tags.contains(params.tag)) {\
                          ctx._source.tags.add(params.tag);\
                      }";

        self.indices
            .post
            .update_doc(
                post,
                json!({
                    "script": {
                        "lang": "painless",
                        "params": { "tag": tag },
                        "source": script,
                    }
                }),
            )
            .await
    }

    pub async fn add_tag_alias(&self, id: Uuid, alias: &str) -> Result<()> {
        self.add_entity_alias(&self.indices.tag, id, alias).await
    }

    pub async fn add_user_alias(&self, id: Uuid, alias: &str) -> Result<()> {
        self.add_entity_alias(&self.indices.user, id, alias).await
    }

    pub async fn delete_entity_alias(
        &self,
        index: &Index,
        id: Uuid,
        alias: &str,
    ) -> Result<()> {
        let script = "if (ctx._source.names.contains(params.alias)) {\
                          ctx._source.names.remove(\
                              ctx._source.names.indexOf(params.alias)\
                          );\
                      }";

        index
            .update_doc(
                id,
                json!({
                    "script": {
                        "lang": "painless",
                        "params": { "alias": alias },
                        "source": script,
                    }
                }),
            )
            .await
    }

    pub async fn delete_post(&self, post: Uuid) -> Result<()> {
        self.indices.post.delete_doc(post).await
    }

    pub async fn find_entities(
        &self,
        index: &Index,
        query: &ProfileQuery,
    ) -> Result<SearchResult<Uuid>> {
        index.search(json!({
            "_source": false,
            "from": query.pagination.from,
            "size": query.pagination.size,
            "query": {
                "bool": {
                    "must": {
                        "multi_match": {
                            "query": query.name,
                            "type": "bool_prefix",
                            "fields": ["names", "names._2gram", "names._3gram"]
                        }
                    },
                    "must_not": {
                        "ids": {
                            "values": query.exclude
                        }
                    }
                }
            }
        })).await
    }

    pub async fn find_posts(
        &self,
        query: &PostQuery,
    ) -> Result<SearchResult<Uuid>> {
        let mut filter: Vec<Json> = vec![json!({
            "term": {
                "visibility": query.visibility
            }
        })];

        if let Some(poster) = query.poster {
            filter.push(json!({
                "term": {
                    "poster": {
                        "value": poster
                    }
                }
            }))
        }

        if !query.tags.is_empty() {
            filter.push(json!({
                "terms_set": {
                    "tags": {
                        "terms": query.tags,
                        "minimum_should_match_script": {
                            "source": query.tags.len().to_string()
                        }
                    }
                }
            }));
        }

        let mut bool = Map::new();

        bool.insert("filter".into(), Json::Array(filter));

        if !query.text.is_empty() {
            bool.insert(
                "must".into(),
                json!({
                    "multi_match": {
                        "query": query.text,
                        "fields": ["title^3", "description"]
                    }
                }),
            );
        }

        let sort = match query.sort.value {
            minty::PostSortValue::Created => "created",
            minty::PostSortValue::Modified => "modified",
            minty::PostSortValue::Relevance => "_score",
            minty::PostSortValue::Title => "title.keyword",
        };

        let query = json!({
            "_source": false,
            "from": query.pagination.from,
            "size": query.pagination.size,
            "query": {
                "bool": bool
            },
            "sort": {
                sort: query.sort.order
            }
        });

        self.indices.post.search(query).await
    }

    pub async fn publish_post(
        &self,
        post: Uuid,
        timestamp: DateTime,
    ) -> Result<()> {
        self.indices
            .post
            .update_doc(
                post,
                json!({
                    "doc": {
                        "visibility": Visibility::Public,
                        "created": timestamp,
                        "modified": timestamp
                    }
                }),
            )
            .await
    }

    pub async fn remove_post_tag(&self, post: Uuid, tag: Uuid) -> Result<()> {
        let script = "if (ctx._source.tags.contains(params.tag)) {\
                          ctx._source.tags.remove(\
                              ctx._source.tags.indexOf(params.tag)\
                          );\
                      }";

        self.indices
            .post
            .update_doc(
                post,
                json!({
                    "script": {
                        "lang": "painless",
                        "params": { "tag": tag },
                        "source": script
                    }
                }),
            )
            .await
    }

    pub async fn update_entity_name(
        &self,
        id: Uuid,
        old: &str,
        new: &str,
        index: &Index,
    ) -> Result<()> {
        let script = "if (ctx._source.names.contains(params.old)) {\
                          ctx._source.names.remove(\
                              ctx._source.names.indexOf(params.old)\
                          );\
                          ctx._source.names.add(params.new);\
                      }";

        index
            .update_doc(
                id,
                json!({
                    "script": {
                        "lang": "painless",
                        "params": {
                            "old": old,
                            "new": new,
                        },
                        "source": script
                    }
                }),
            )
            .await
    }

    pub async fn update_post_modified(
        &self,
        post: Uuid,
        modified: DateTime,
    ) -> Result<()> {
        self.indices
            .post
            .update_doc(
                post,
                json!({
                    "doc": { "modified": modified }
                }),
            )
            .await
    }

    pub async fn update_post_description(
        &self,
        post: Uuid,
        description: &str,
        modified: DateTime,
    ) -> Result<()> {
        self.indices
            .post
            .update_doc(
                post,
                json!({
                    "doc": {
                        "description": description,
                        "modified": modified
                    }
                }),
            )
            .await
    }

    pub async fn update_post_title(
        &self,
        post: Uuid,
        title: &str,
        modified: DateTime,
    ) -> Result<()> {
        self.indices
            .post
            .update_doc(
                post,
                json!({
                    "doc": {
                        "title": title,
                        "modified": modified
                    }
                }),
            )
            .await
    }
}
