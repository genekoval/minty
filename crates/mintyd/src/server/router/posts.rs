use super::{AppState, Result, Router};

use axum::{
    extract::{Query, State},
    routing::get,
    Json,
};
use minty::{
    PostPreview, PostQuery, PostSort, PostSortValue, SearchResult, SortOrder,
    Uuid, Visibility,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Params {
    from: Option<u32>,
    size: Option<u32>,
    q: Option<String>,
    tags: Option<Vec<Uuid>>,
    vis: Option<Visibility>,
    sort: Option<PostSortValue>,
    order: Option<SortOrder>,
}

impl From<Params> for PostQuery {
    fn from(
        Params {
            from,
            size,
            q,
            tags,
            vis,
            sort,
            order,
        }: Params,
    ) -> Self {
        let sort_value = sort.unwrap_or_default();

        Self {
            from: from.unwrap_or(0),
            size: size.unwrap_or(10),
            text: q,
            tags: tags.unwrap_or_default(),
            visibility: vis.unwrap_or_default(),
            sort: PostSort {
                value: sort_value,
                order: order.unwrap_or(sort_value.default_order()),
            },
        }
    }
}

async fn get_posts(
    State(AppState { repo }): State<AppState>,
    Query(query): Query<Params>,
) -> Result<Json<SearchResult<PostPreview>>> {
    Ok(Json(repo.get_posts(&query.into()).await?))
}

pub fn routes() -> Router {
    Router::new().route("/", get(get_posts))
}
