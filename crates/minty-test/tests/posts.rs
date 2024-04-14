use minty_test::{posts::*, repo, tags::LANGUAGES};

use minty::{
    Pagination, PostQuery, PostSort, PostSortValue::*, Repo, SortOrder::*,
};
use tokio::test;
use uuid::Uuid;

#[test]
async fn limit_results() {
    find(
        PostQuery {
            pagination: Pagination { from: 0, size: 3 },
            sort: PostSort::TITLE,
            ..Default::default()
        },
        [C, CPP, JAVA],
    )
    .await;
}

#[test]
async fn search_title() {
    find_text("java", [JAVA]).await;
    find_text("c", [C, CPP]).await;
}

#[test]
async fn search_description() {
    find_text("programming language", [C, CPP, JAVA, JS]).await;
    find_text("html", [JS]).await;
}

#[test]
async fn sort_created_ascending() {
    find(
        PostQuery {
            sort: PostSort {
                value: Created,
                order: Ascending,
            },
            ..Default::default()
        },
        [C, CPP, JAVA, JS, RUST],
    )
    .await;
}

#[test]
async fn sort_created_descending() {
    find(
        PostQuery {
            sort: PostSort {
                value: Created,
                order: Descending,
            },
            ..Default::default()
        },
        [RUST, JS, JAVA, CPP, C],
    )
    .await;
}

#[test]
async fn sort_title_ascending() {
    find(
        PostQuery {
            sort: PostSort {
                value: Title,
                order: Ascending,
            },
            ..Default::default()
        },
        [C, CPP, JAVA, JS, RUST],
    )
    .await;
}

#[test]
async fn sort_title_descending() {
    find(
        PostQuery {
            sort: PostSort {
                value: Title,
                order: Descending,
            },
            ..Default::default()
        },
        [RUST, JS, JAVA, CPP, C],
    )
    .await;
}

async fn find<T>(mut query: PostQuery, expected: T)
where
    T: AsRef<[Uuid]>,
{
    let expected = expected.as_ref();
    let repo = repo();

    query.tags.push(LANGUAGES);

    let result = repo.get_posts(&query).await.unwrap();
    assert_eq!(expected.len(), result.hits.len());

    let hits: Vec<_> = result.hits.iter().map(|hit| hit.id).collect();
    assert_eq!(expected, hits);
}

async fn find_text<T>(text: &str, expected: T)
where
    T: AsRef<[Uuid]>,
{
    find(
        PostQuery {
            text: text.into(),
            sort: PostSort::TITLE,
            ..Default::default()
        },
        expected,
    )
    .await;
}
