use minty_test::{not_found, repo};

use minty::{
    text::{Description, Name},
    Pagination, ProfileQuery, Repo, Url,
};
use tokio::test;
use uuid::uuid;

#[test]
async fn add_tag() {
    const NAME: &str = "Minty Test";

    let repo = repo().await;
    let name = Name::new(NAME).unwrap();
    let id = repo.add_tag(name).await.unwrap();
    let tag = repo.get_tag(id).await.unwrap();

    assert_eq!(NAME, tag.profile.name);
}

#[test]
async fn add_tag_alias() {
    const NAME: &str = "Tag Name";
    const ALIAS: &str = "Tag Alias";

    let repo = repo().await;

    let name = Name::new(NAME).unwrap();
    let id = repo.add_tag(name.clone()).await.unwrap();

    let alias = Name::new(ALIAS).unwrap();

    for _ in 0..2 {
        let tag = repo.add_tag_alias(id, alias.clone()).await.unwrap();

        assert_eq!(tag.name, NAME);
        assert_eq!(tag.aliases.len(), 1);
        assert_eq!(tag.aliases.first().unwrap(), ALIAS);
    }

    let tag = repo.add_tag_alias(id, name).await.unwrap();

    assert_eq!(tag.name, NAME);
    assert_eq!(tag.aliases.len(), 1);
    assert_eq!(tag.aliases.first().unwrap(), ALIAS);

    let tag = repo.get_tag(id).await.unwrap();

    assert_eq!(tag.profile.name, NAME);
    assert_eq!(tag.profile.aliases.len(), 1);
    assert_eq!(tag.profile.aliases.first().unwrap(), ALIAS);

    let id = uuid!("4b88efa9-e961-4e63-8142-5c39e289a29a");
    not_found!(repo.add_tag_alias(id, alias).await, "tag", id);
}

#[test]
async fn add_tag_source() {
    const SOURCE: &str = "https://example.com/hello";

    let repo = repo().await;
    let name = Name::new("Tag Name").unwrap();
    let id = repo.add_tag(name).await.unwrap();
    let url = Url::parse(SOURCE).unwrap();
    let source = repo.add_tag_source(id, &url).await.unwrap();

    assert_eq!(url, source.url);
    assert!(source.icon.is_none());

    let tag = repo.get_tag(id).await.unwrap();

    assert_eq!(1, tag.profile.sources.len());

    let result = tag.profile.sources.first().unwrap();

    assert_eq!(source.id, result.id);
    assert_eq!(source.url, result.url);
    assert_eq!(source.icon, result.icon);

    let id = uuid!("b70802c1-3c9c-40ea-ae71-5e9dd8917936");
    not_found!(repo.add_tag_source(id, &url).await, "tag", id);
}

#[test]
async fn delete_tag() {
    let repo = repo().await;
    let name = Name::new("Delete Me").unwrap();
    let id = repo.add_tag(name).await.unwrap();
    repo.delete_tag(id).await.unwrap();
    not_found!(repo.delete_tag(id).await, "tag", id);
}

#[test]
async fn delete_tag_alias() {
    const NAME: &str = "Tag Name";
    const ALIAS: &str = "Delete Me";

    let repo = repo().await;
    let name = Name::new(NAME).unwrap();
    let id = repo.add_tag(name).await.unwrap();
    let alias = Name::new(ALIAS).unwrap();
    repo.add_tag_alias(id, alias).await.unwrap();

    for _ in 0..2 {
        let tag = repo.delete_tag_alias(id, ALIAS).await.unwrap();
        assert_eq!(NAME, tag.name);
        assert!(tag.aliases.is_empty());

        let tag = repo.get_tag(id).await.unwrap();
        assert_eq!(NAME, tag.profile.name);
        assert!(tag.profile.aliases.is_empty());
    }

    let id = uuid!("010bf31b-eb5d-4f35-8314-678abbd6cd36");
    not_found!(repo.delete_tag_alias(id, ALIAS).await, "tag", id);
}

#[test]
async fn delete_tag_source() {
    let repo = repo().await;
    let name = Name::new("Tag Name").unwrap();
    let id = repo.add_tag(name).await.unwrap();
    let url = Url::parse("https://example.com/hello").unwrap();
    let source = repo.add_tag_source(id, &url).await.unwrap();

    repo.delete_tag_source(id, source.id).await.unwrap();

    not_found!(
        repo.delete_tag_source(id, source.id).await,
        "tag or source not found"
    );
}

#[test]
async fn delete_tag_sources() {
    const HOST: &str = "example.com";

    let repo = repo().await;
    let name = Name::new("Tag Name").unwrap();
    let id = repo.add_tag(name).await.unwrap();

    for path in ["hello/world", "foo/bar"] {
        let url = format!("https://{HOST}");
        let mut url = Url::parse(&url).unwrap();
        url.set_path(path);
        repo.add_tag_source(id, &url).await.unwrap();
    }

    repo.delete_tag_sources(id, &[HOST.to_owned()])
        .await
        .unwrap();

    let tag = repo.get_tag(id).await.unwrap();

    assert!(
        tag.profile.sources.is_empty(),
        "tag.sources = {:?}",
        tag.profile.sources
    );
}

#[test]
async fn get_tags() {
    let repo = repo().await;
    let java = repo.add_tag(Name::new("Java").unwrap()).await.unwrap();
    let js = repo
        .add_tag(Name::new("JavaScript").unwrap())
        .await
        .unwrap();

    let mut query = ProfileQuery {
        pagination: Pagination {
            from: 0,
            size: 1_000,
        },
        name: "java".into(),
        exclude: Default::default(),
    };

    let mut result = repo.get_tags(&query).await.unwrap();

    assert!(result.total >= 2, "result.total = {}", result.total);

    let mut hits: Vec<_> = result.hits.iter().map(|hit| hit.id).collect();

    assert!(hits.contains(&java));
    assert!(hits.contains(&js));

    query.name = "javas".into();
    result = repo.get_tags(&query).await.unwrap();
    hits = result.hits.iter().map(|hit| hit.id).collect();

    assert!(hits.contains(&js));
    assert!(!hits.contains(&java));
}

#[test]
async fn set_tag_description() {
    const NAME: &str = "Tag Name";
    const DESCRIPTION: &str = "A description of a tag.";

    let repo = repo().await;
    let name = Name::new(NAME).unwrap();
    let id = repo.add_tag(name).await.unwrap();

    let description = Description::new(DESCRIPTION).unwrap();
    let mut result = repo
        .set_tag_description(id, description.clone())
        .await
        .unwrap();
    assert_eq!(result, DESCRIPTION);

    result = repo.get_tag(id).await.unwrap().profile.description;
    assert_eq!(result, DESCRIPTION);

    let id = uuid!("43b59116-2faa-4a15-9ae8-bb27c11183ab");
    not_found!(repo.set_tag_description(id, description).await, "tag", id);
}

#[test]
async fn set_tag_name() {
    const NAME: &str = "Tag Name";
    const ALIAS: &str = "Tag Alias";

    let repo = repo().await;

    let name = Name::new(NAME).unwrap();
    let id = repo.add_tag(name.clone()).await.unwrap();

    let alias = Name::new(ALIAS).unwrap();
    let tag = repo.set_tag_name(id, alias).await.unwrap();

    assert_eq!(tag.name, ALIAS);
    assert!(tag.aliases.is_empty());

    repo.add_tag_alias(id, name.clone()).await.unwrap();
    let tag = repo.set_tag_name(id, name.clone()).await.unwrap();

    assert_eq!(tag.name, NAME);
    assert_eq!(tag.aliases.len(), 1);
    assert_eq!(tag.aliases.first().unwrap(), ALIAS);

    let tag = repo.get_tag(id).await.unwrap();

    assert_eq!(tag.profile.name, NAME);
    assert_eq!(tag.profile.aliases.len(), 1);
    assert_eq!(tag.profile.aliases.first().unwrap(), ALIAS);

    let id = uuid!("90ed1a19-6b7f-4892-b096-5280dcc652d6");
    not_found!(repo.set_tag_name(id, name.clone()).await, "tag", id);
}
