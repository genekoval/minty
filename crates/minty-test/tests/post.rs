use minty_test::{admin, not_found, objects, posts, tags, users};

use minty::{
    text::{Description, Name, PostTitle},
    ErrorKind, Post, PostParts, Repo, Uuid, Visibility,
};
use tokio::test;
use uuid::uuid;

#[test]
async fn add_post_tag() {
    let repo = admin().await;

    let mut id = uuid!("b53b3334-4448-433d-bcb7-de424be0cf06");
    let mut tag = uuid!("e1db9c45-5993-42bf-b1be-29373baef6bf");

    not_found!(repo.add_post_tag(id, tag).await, "post", id);

    id = repo.create_post(&Default::default()).await.unwrap();

    not_found!(repo.add_post_tag(id, tag).await, "tag", tag);

    tag = repo.add_tag(Name::new("Test Tag").unwrap()).await.unwrap();

    for _ in 0..2 {
        repo.add_post_tag(id, tag).await.unwrap();
        let tags = repo.get_post(id).await.unwrap().tags;
        assert_eq!(tags.len(), 1);
        assert_eq!(tags.first().unwrap().id, tag);

        assert_eq!(repo.get_tag(tag).await.unwrap().post_count, 1);
    }
}

#[test]
async fn add_related_post() {
    let repo = admin().await;

    let mut id = uuid!("16357a95-02ea-457d-895f-de46cf06a1ec");
    let mut related = uuid!("fb04b233-a059-4619-8401-c8d06aa4aee9");

    not_found!(repo.add_related_post(id, related).await, "post", id);

    id = repo.create_post(&Default::default()).await.unwrap();

    not_found!(repo.add_related_post(id, related).await, "post", related);

    related = posts::BUNNY;

    for _ in 0..2 {
        repo.add_related_post(id, related).await.unwrap();
        let posts = repo.get_post(id).await.unwrap().posts;
        assert_eq!(posts.len(), 1);
        assert_eq!(posts.first().unwrap().id, related);
    }

    let err = repo
        .add_related_post(id, id)
        .await
        .expect_err("post cannot be related to itself");
    match err.kind() {
        ErrorKind::Client => (),
        _ => panic!("unexpected error: {err:?}"),
    }
}

#[test]
async fn append_post_objects() {
    use objects::*;

    let repo = admin().await;

    let id = repo.create_post(&Default::default()).await.unwrap();
    let mut post = repo.get_post(id).await.unwrap();
    let modified = repo.append_post_objects(id, &[BUNNY]).await.unwrap();
    assert!(modified > post.modified);

    post = repo.get_post(id).await.unwrap();

    assert_eq!(modified, post.modified);
    assert_eq!(post.objects.len(), 1);
    assert_eq!(post.objects.first().unwrap().id, BUNNY);

    repo.append_post_objects(id, &[SAND]).await.unwrap();
    let objects: Vec<_> = repo
        .get_post(id)
        .await
        .unwrap()
        .objects
        .into_iter()
        .map(|o| o.id)
        .collect();
    assert_eq!(objects, &[BUNNY, SAND]);

    repo.append_post_objects(id, &[BUNNY]).await.unwrap();
    let objects: Vec<_> = repo
        .get_post(id)
        .await
        .unwrap()
        .objects
        .into_iter()
        .map(|o| o.id)
        .collect();
    assert_eq!(objects, &[SAND, BUNNY]);
}

#[test]
async fn create_post() {
    const TITLE: &str = "My Test Post";
    const DESCRIPTION: &str = "A test description.";
    const OBJECT: Uuid = objects::BUNNY;
    const POST: Uuid = posts::BUNNY;
    const TAG: Uuid = tags::VIDEOS;

    let repo = admin().await;

    let post_id = repo
        .create_post(&PostParts {
            title: Some(PostTitle::new(TITLE).unwrap()),
            description: Some(Description::new(DESCRIPTION).unwrap()),
            visibility: Some(Visibility::Public),
            objects: Some(vec![OBJECT]),
            posts: Some(vec![POST]),
            tags: Some(vec![TAG]),
        })
        .await
        .unwrap();

    let Post {
        id,
        poster,
        title,
        description,
        visibility,
        created,
        modified,
        objects,
        posts,
        tags,
        comment_count,
    } = repo.get_post(post_id).await.unwrap();

    assert_eq!(id, post_id);
    assert_eq!(poster.map(|user| user.id), Some(users::MINTY));
    assert_eq!(title, TITLE);
    assert_eq!(description, DESCRIPTION);
    assert_eq!(Visibility::Public, visibility);
    assert_eq!(created, modified);
    assert_eq!(comment_count, 0);

    assert_eq!(objects.len(), 1);
    assert_eq!(objects.first().map(|obj| obj.id), Some(OBJECT));

    assert_eq!(posts.len(), 1);
    assert_eq!(posts.first().map(|post| post.id), Some(POST));

    assert_eq!(tags.len(), 1);
    assert_eq!(tags.first().map(|tag| tag.id), Some(TAG));
}

#[test]
async fn delete_post() {
    let repo = admin().await;

    let id = repo
        .create_post(&PostParts {
            title: Some(PostTitle::new("Delete Me").unwrap()),
            ..Default::default()
        })
        .await
        .unwrap();

    repo.delete_post(id).await.unwrap();

    not_found!(repo.delete_post(id).await, "post", id);
}

#[test]
async fn delete_post_objects() {
    use objects::*;

    const OBJECTS: [Uuid; 2] = [BUNNY, SAND];

    let repo = admin().await;

    for count in 1..=2 {
        let id = repo
            .create_post(&PostParts {
                objects: Some(OBJECTS.to_vec()),
                ..Default::default()
            })
            .await
            .unwrap();
        let mut post = repo.get_post(id).await.unwrap();

        let modified = repo
            .delete_post_objects(id, &OBJECTS[..count])
            .await
            .unwrap();

        assert!(modified > post.modified);

        post = repo.get_post(id).await.unwrap();
        assert_eq!(modified, post.modified);

        let result: Vec<_> = post.objects.iter().map(|o| o.id).collect();
        assert_eq!(result, &OBJECTS[count..]);
    }
}

#[test]
async fn delete_post_tag() {
    let repo = admin().await;

    let tag = repo.add_tag(Name::new("Test Tag").unwrap()).await.unwrap();
    let id = repo
        .create_post(&PostParts {
            tags: Some(vec![tag]),
            ..Default::default()
        })
        .await
        .unwrap();

    repo.delete_post_tag(id, tag).await.unwrap();

    assert!(repo.get_post(id).await.unwrap().tags.is_empty());
    assert_eq!(repo.get_tag(tag).await.unwrap().post_count, 0);

    not_found!(repo.delete_post_tag(id, tag).await, "tag was removed");

    assert_eq!(repo.get_tag(tag).await.unwrap().post_count, 0);
}

#[test]
async fn delete_related_post() {
    let repo = admin().await;

    let related = posts::BUNNY;
    let id = repo
        .create_post(&PostParts {
            posts: Some(vec![related]),
            ..Default::default()
        })
        .await
        .unwrap();

    repo.delete_related_post(id, related).await.unwrap();
    assert!(repo.get_post(id).await.unwrap().posts.is_empty());

    not_found!(
        repo.delete_related_post(id, related).await,
        "related post was removed"
    );
}

#[test]
async fn insert_post_objects() {
    use objects::*;

    let repo = admin().await;

    let id = repo
        .create_post(&PostParts {
            objects: Some(vec![BUNNY]),
            ..Default::default()
        })
        .await
        .unwrap();
    let mut post = repo.get_post(id).await.unwrap();
    let modified = repo.insert_post_objects(id, &[SAND], BUNNY).await.unwrap();
    assert!(modified > post.modified);

    post = repo.get_post(id).await.unwrap();

    assert_eq!(modified, post.modified);

    let objects: Vec<_> = post.objects.iter().map(|o| o.id).collect();
    assert_eq!(objects, &[SAND, BUNNY]);
}

#[test]
async fn publish_post() {
    let repo = admin().await;
    let id = repo
        .create_post(&PostParts {
            visibility: Some(Visibility::Draft),
            ..Default::default()
        })
        .await
        .unwrap();
    let draft = repo.get_post(id).await.unwrap();

    assert_eq!(draft.visibility, Visibility::Draft);

    let title = PostTitle::new("Publishing a Draft").unwrap();
    let modified = repo.set_post_title(id, title).await.unwrap().date_modified;

    repo.publish_post(id).await.unwrap();

    let post = repo.get_post(id).await.unwrap();

    assert_eq!(post.visibility, Visibility::Public);
    assert_eq!(post.created, post.modified);
    assert!(post.created > draft.created);
    assert!(post.created > modified);
}

#[test]
async fn set_post_description() {
    const DESCRIPTION: &str = "Test description";

    let repo = admin().await;
    let id = repo.create_post(&Default::default()).await.unwrap();
    let description = Description::new(DESCRIPTION).unwrap();
    let update = repo.set_post_description(id, description).await.unwrap();
    let post = repo.get_post(id).await.unwrap();

    assert_eq!(update.new_value, DESCRIPTION);
    assert_eq!(post.description, DESCRIPTION);
    assert_eq!(post.modified, update.date_modified);
    assert!(post.created < post.modified);
}

#[test]
async fn set_post_title() {
    const TITLE: &str = "Test title";

    let repo = admin().await;
    let id = repo.create_post(&Default::default()).await.unwrap();
    let title = PostTitle::new(TITLE).unwrap();
    let update = repo.set_post_title(id, title).await.unwrap();
    let post = repo.get_post(id).await.unwrap();

    assert_eq!(update.new_value, TITLE);
    assert_eq!(post.title, TITLE);
    assert_eq!(post.modified, update.date_modified);
    assert!(post.created < post.modified);
}
