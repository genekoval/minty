use crate::model::{PostSort, PostSortValue, SortOrder, Uuid, Visibility};

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

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

    #[serde(
        serialize_with = "serialize_id_list",
        deserialize_with = "deserialize_id_list"
    )]
    pub tags: Option<Vec<Uuid>>,

    pub vis: Option<Visibility>,

    #[serde(serialize_with = "PostQuery::serialize_sort")]
    pub sort: Option<PostSort>,

    pub order: Option<SortOrder>,
}

impl PostQuery {
    fn serialize_sort<S: Serializer>(
        sort: &Option<PostSort>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match sort {
            Some(sort) => sort.value.serialize(serializer),
            None => serializer.serialize_none(),
        }
    }
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
        let mut sort = sort.unwrap_or_default();

        if let Some(order) = order {
            sort.order = order;
        }

        Self {
            pagination: Pagination { from, size }.into(),
            poster: u,
            text: q.unwrap_or_default(),
            tags: tags.unwrap_or_default(),
            visibility: vis.unwrap_or_default(),
            sort,
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
            tags: (!tags.is_empty()).then_some(tags),
            vis: if visibility != Visibility::default() {
                Some(visibility)
            } else {
                None
            },
            sort: if sort.value != PostSortValue::default() {
                Some(sort.value.into())
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

    #[serde(
        serialize_with = "serialize_id_list",
        deserialize_with = "deserialize_id_list"
    )]
    pub exclude: Option<Vec<Uuid>>,
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
            exclude: exclude.unwrap_or_default(),
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
            exclude: (!exclude.is_empty()).then_some(exclude),
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

fn serialize_id_list<S>(
    list: &Option<Vec<Uuid>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let Some(list) = list else {
        return serializer.serialize_none();
    };

    let mut string = String::new();

    for (i, id) in list.iter().enumerate() {
        string.push_str(&id.to_string());

        if i < list.len() - 1 {
            string.push(',');
        }
    }

    string.serialize(serializer)
}

fn deserialize_id_list<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<Uuid>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<String> = Option::deserialize(deserializer)?;

    let Some(string) = value else { return Ok(None) };

    let list = string
        .split(',')
        .map(Uuid::parse_str)
        .collect::<Result<Vec<_>, _>>()
        .map_err(de::Error::custom)?;

    Ok(Some(list))
}
