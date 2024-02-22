use crate::model::{PostSort, PostSortValue, SortOrder, Uuid, Visibility};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Pagination {
    pub from: Option<u32>,
    pub size: Option<u32>,
}

impl From<Pagination> for crate::Pagination {
    fn from(Pagination { from, size }: Pagination) -> Self {
        Self {
            from: from.unwrap_or_default(),
            size: size.unwrap_or(crate::Pagination::default_size()),
        }
    }
}

impl From<crate::Pagination> for Pagination {
    fn from(crate::Pagination { from, size }: crate::Pagination) -> Self {
        Self {
            from: if from > 0 { Some(from) } else { None },
            size: if size != crate::Pagination::default_size() {
                Some(size)
            } else {
                None
            },
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct PostQuery {
    #[serde(flatten)]
    pub pagination: Pagination,
    pub q: Option<String>,
    pub tags: Option<String>,
    pub vis: Option<Visibility>,
    pub sort: Option<PostSortValue>,
    pub order: Option<SortOrder>,
}

impl From<PostQuery> for crate::PostQuery {
    fn from(
        PostQuery {
            pagination,
            q,
            tags,
            vis,
            sort,
            order,
        }: PostQuery,
    ) -> Self {
        let sort_value = sort.unwrap_or_default();

        Self {
            pagination: pagination.into(),
            text: q.unwrap_or_default(),
            tags: tags
                .map(|tags| {
                    tags.split(',')
                        .map(|tag| Uuid::parse_str(tag).unwrap())
                        .collect()
                })
                .unwrap_or_default(),
            visibility: vis.unwrap_or_default(),
            sort: PostSort {
                value: sort_value,
                order: order.unwrap_or(sort_value.default_order()),
            },
        }
    }
}

impl From<crate::PostQuery> for PostQuery {
    fn from(
        crate::PostQuery {
            pagination,
            text,
            tags,
            visibility,
            sort,
        }: crate::PostQuery,
    ) -> Self {
        Self {
            pagination: pagination.into(),
            q: {
                let text = text.trim();
                if text.is_empty() {
                    None
                } else {
                    Some(text.into())
                }
            },
            tags: if tags.is_empty() {
                None
            } else {
                let tags = tags
                    .into_iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<String>>()
                    .join(",");

                Some(tags)
            },
            vis: if visibility != Visibility::default() {
                Some(visibility)
            } else {
                None
            },
            sort: if sort.value != PostSortValue::default() {
                Some(sort.value)
            } else {
                None
            },
            order: if sort.order != sort.value.default_order() {
                Some(sort.order)
            } else {
                None
            },
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TagQuery {
    #[serde(flatten)]
    pub pagination: Pagination,
    pub name: String,
    pub exclude: Option<String>,
}

impl From<TagQuery> for crate::TagQuery {
    fn from(
        TagQuery {
            pagination,
            name,
            exclude,
        }: TagQuery,
    ) -> Self {
        Self {
            pagination: pagination.into(),
            name,
            exclude: exclude
                .map(|tags| {
                    tags.split(',')
                        .map(|tag| Uuid::parse_str(tag).unwrap())
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}

impl From<crate::TagQuery> for TagQuery {
    fn from(
        crate::TagQuery {
            pagination,
            name,
            exclude,
        }: crate::TagQuery,
    ) -> Self {
        Self {
            pagination: pagination.into(),
            name,
            exclude: if exclude.is_empty() {
                None
            } else {
                let tags = exclude
                    .into_iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<String>>()
                    .join(",");

                Some(tags)
            },
        }
    }
}
