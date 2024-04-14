mod index;
mod response;

pub use index::{Index, Indices};

use response::ResponseExt;

use crate::{conf::SearchConfig, db::PostSearch, Result};

use elasticsearch::{
    http::transport::{SingleNodeConnectionPool, TransportBuilder},
    Elasticsearch,
};
use minty::{DateTime, PostQuery, SearchResult, TagQuery, Uuid, Visibility};
use serde_json::json;
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

    pub async fn add_tag_alias(&self, tag: Uuid, alias: &str) -> Result<()> {
        let script = "if (!ctx._source.names.contains(params.alias)) {\
                          ctx._source.names.add(params.alias);\
                      }";

        self.indices
            .tag
            .update_doc(
                tag,
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

    pub async fn delete_post(&self, post: Uuid) -> Result<()> {
        self.indices.post.delete_doc(post).await
    }

    pub async fn delete_tag(&self, tag: Uuid) -> Result<()> {
        self.indices.tag.delete_doc(tag).await
    }

    pub async fn delete_tag_alias(&self, tag: Uuid, alias: &str) -> Result<()> {
        let script = "if (ctx._source.names.contains(params.alias)) {\
                          ctx._source.names.remove(\
                              ctx._source.names.indexOf(params.alias)\
                          );\
                      }";

        self.indices
            .tag
            .update_doc(
                tag,
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

    pub async fn find_posts(
        &self,
        query: &PostQuery,
    ) -> Result<SearchResult<Uuid>> {
        let mut q = json!({
            "_source": false,
            "from": query.pagination.from,
            "size": query.pagination.size,
            "query": {
                "bool": {
                    "filter": [
                        {
                            "term": {
                                "visibility": query.visibility
                            }
                        }
                    ]
                }
            }
        });

        if !query.text.is_empty() || !query.tags.is_empty() {
            let b = q
                .get_mut("query")
                .unwrap()
                .get_mut("bool")
                .unwrap()
                .as_object_mut()
                .unwrap();

            if !query.text.is_empty() {
                b.insert(
                    "must".into(),
                    json!({
                        "multi_match": {
                            "query": query.text,
                            "fields": ["title^3", "description"]
                        }
                    }),
                );
            }

            if !query.tags.is_empty() {
                b.get_mut("filter").unwrap().as_array_mut().unwrap().push(
                    json!({
                        "terms_set": {
                            "tags": {
                                "terms": query.tags,
                                "minimum_should_match_script": {
                                    "source": query.tags.len().to_string()
                                }
                            }
                        }

                    }),
                );
            }
        }

        let value = match query.sort.value {
            minty::PostSortValue::Created => "created",
            minty::PostSortValue::Modified => "modified",
            minty::PostSortValue::Relevance => "_score",
            minty::PostSortValue::Title => "title.keyword",
        };

        q.as_object_mut()
            .unwrap()
            .insert("sort".into(), json!({ value: query.sort.order }));

        self.indices.post.search(q).await
    }

    pub async fn find_tags(
        &self,
        query: &TagQuery,
    ) -> Result<SearchResult<Uuid>> {
        self.indices.tag.search(json!({
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

    pub async fn update_tag_name(
        &self,
        tag: Uuid,
        old: &str,
        new: &str,
    ) -> Result<()> {
        let script = "if (ctx._source.names.contains(params.old)) {\
                          ctx._source.names.remove(\
                              ctx._source.names.indexOf(params.old)\
                          );\
                          ctx._source.names.add(params.new);\
                      }";

        self.indices
            .tag
            .update_doc(
                tag,
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
}
