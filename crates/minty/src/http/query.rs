use crate::model::{PostSort, PostSortValue, SortOrder, Uuid, Visibility};

use serde::{Deserialize, Serialize};

pub trait QueryParams: Sized {
    type Params: From<Self> + Serialize;

    fn into_query_params(self) -> String {
        let params: Self::Params = self.into();

        serde_urlencoded::to_string(&params)
            .expect("query serialization should always succeed")
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct DeleteComment {
    pub recursive: Option<bool>,
}

struct Pagination {
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
    pub from: Option<u32>,
    pub size: Option<u32>,
    pub u: Option<Uuid>,
    pub q: Option<String>,
    pub tags: Option<String>,
    pub vis: Option<Visibility>,
    pub sort: Option<PostSortValue>,
    pub order: Option<SortOrder>,
}

impl From<PostQuery> for crate::PostQuery {
    fn from(
        PostQuery {
            from,
            size,
            u,
            q,
            tags,
            vis,
            sort,
            order,
        }: PostQuery,
    ) -> Self {
        let sort_value = sort.unwrap_or_default();

        Self {
            pagination: Pagination { from, size }.into(),
            poster: u,
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
            poster,
            text,
            tags,
            visibility,
            sort,
        }: crate::PostQuery,
    ) -> Self {
        let Pagination { from, size } = pagination.into();

        Self {
            from,
            size,
            u: poster,
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

impl QueryParams for crate::PostQuery {
    type Params = PostQuery;
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ProfileQuery {
    pub from: Option<u32>,
    pub size: Option<u32>,
    pub name: String,
    pub exclude: Option<String>,
}

impl From<ProfileQuery> for crate::ProfileQuery {
    fn from(
        ProfileQuery {
            from,
            size,
            name,
            exclude,
        }: ProfileQuery,
    ) -> Self {
        Self {
            pagination: Pagination { from, size }.into(),
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

impl From<crate::ProfileQuery> for ProfileQuery {
    fn from(
        crate::ProfileQuery {
            pagination,
            name,
            exclude,
        }: crate::ProfileQuery,
    ) -> Self {
        let Pagination { from, size } = pagination.into();

        Self {
            from,
            size,
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

impl QueryParams for crate::ProfileQuery {
    type Params = ProfileQuery;
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct SetProfileName {
    pub main: Option<bool>,
}

impl SetProfileName {
    pub fn main(value: bool) -> Self {
        Self { main: Some(value) }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SignUp {
    pub invitation: Option<String>,
}
