use minty_test::{not_found, posts, repo};

use minty::{text, Repo};
use tokio::test;
use uuid::{uuid, Uuid};

const COMMENT: Uuid = uuid!("76c9699d-9b69-4857-97d8-31afbe1a00c3");
const POST: Uuid = posts::BUNNY;

#[test]
async fn add_comment() {
    const CONTENT: &str = "A top-level comment.";

    let repo = repo().await;
    let content = text::Comment::new(CONTENT).unwrap();
    let comment = repo.add_comment(POST, content.clone()).await.unwrap();

    assert_eq!(CONTENT, comment.content);
    assert_eq!(0, comment.level);

    let id = uuid!("2956450b-5e6f-461b-9b06-820cd3709375");
    not_found!(repo.add_comment(id, content).await, "post", id);
}

#[test]
async fn add_reply() {
    const CONTENT: &str = "A reply.";

    let repo = repo().await;
    let content = text::Comment::new(CONTENT).unwrap();
    let comment = repo.add_reply(COMMENT, content.clone()).await.unwrap();

    assert_eq!(CONTENT, comment.content);
    assert_eq!(1, comment.level);

    let id = uuid!("51cea605-1f3d-49c7-ac32-83c842e9892d");
    not_found!(repo.add_reply(id, content).await, "comment", id);
}

#[test]
async fn delete_comment() {
    const ROOT: &str = "A root comment.";
    const REPLY: &str = "A reply.";

    let repo = repo().await;
    let build_tree = || async {
        let mut content = text::Comment::new(ROOT).unwrap();
        let comment = repo.add_comment(POST, content).await.unwrap();
        content = text::Comment::new(REPLY).unwrap();
        let reply = repo.add_reply(comment.id, content).await.unwrap();

        (comment, reply)
    };

    let (comment, reply) = build_tree().await;

    for _ in 0..2 {
        repo.delete_comment(comment.id, false).await.unwrap();

        let deleted = repo.get_comment(comment.id).await.unwrap();

        assert_eq!(comment.id, deleted.id);
        assert_eq!(POST, deleted.post_id);
        assert!(deleted.parent_id.is_none());
        assert_eq!(0, deleted.level);
        assert!(
            deleted.content.is_empty(),
            "deleted comments should have no content: {}",
            deleted.content
        );
        assert_eq!(comment.created, deleted.created);
    }

    let reply = repo.get_comment(reply.id).await.unwrap();

    assert_eq!(reply.parent_id, Some(comment.id));
    assert_eq!(REPLY, reply.content);

    repo.delete_comment(reply.id, false).await.unwrap();

    not_found!(repo.get_comment(comment.id).await, "comment", comment.id);
    not_found!(repo.get_comment(reply.id).await, "comment", reply.id);

    let (comment, reply) = build_tree().await;

    repo.delete_comment(comment.id, true).await.unwrap();

    not_found!(repo.get_comment(comment.id).await, "comment", comment.id);
    not_found!(repo.get_comment(reply.id).await, "comment", reply.id);
}

#[test]
async fn get_comment() {
    const CONTENT: &str = "Getting info about a comment.";
    const REPLY: &str = "This is a reply.";

    let repo = repo().await;
    let mut content = text::Comment::new(CONTENT).unwrap();
    let mut data = repo.add_comment(POST, content).await.unwrap();
    let comment = repo.get_comment(data.id).await.unwrap();

    assert_eq!(data.id, comment.id);
    assert_eq!(POST, comment.post_id);
    assert!(comment.parent_id.is_none());
    assert_eq!(0, comment.level);
    assert_eq!(CONTENT, comment.content);
    assert_eq!(data.created, comment.created);

    content = text::Comment::new(REPLY).unwrap();
    data = repo.add_reply(comment.id, content).await.unwrap();
    let reply = repo.get_comment(data.id).await.unwrap();

    assert_eq!(data.id, reply.id);
    assert_eq!(POST, reply.post_id);
    assert_eq!(reply.parent_id, Some(comment.id));
    assert_eq!(1, reply.level);
    assert_eq!(REPLY, reply.content);
    assert_eq!(data.created, reply.created);

    let id = uuid!("c9429ad9-a82a-4062-ac26-3d12ab2ecc8d");
    not_found!(repo.get_comment(id).await, "comment", id);
}

#[test]
async fn get_comments() {
    const POST: Uuid = uuid!("f4d63cb8-46bc-455f-94a0-86476940327a");
    const COMMENTS: [Uuid; 7] = [
        uuid!("483a1e30-9946-4721-b2b0-91dc726526ee"),
        uuid!("db42d7a1-5c81-4684-940c-b37e200da435"),
        uuid!("7b77b44f-1e3a-48cd-8399-6698ef7f2869"),
        uuid!("ef2913a5-96e7-4c40-bc68-b234f8c4ffd5"),
        uuid!("013fcf3f-4df2-4c88-801f-da703e43a1c5"),
        uuid!("c7234c95-4b9e-48f4-9945-64fe882247ae"),
        uuid!("ae993efd-7fb0-4fd5-99a0-253553455f47"),
    ];

    let comments: Vec<_> = repo()
        .await
        .get_comments(POST)
        .await
        .unwrap()
        .into_iter()
        .map(|comment| comment.id)
        .collect();

    assert_eq!(COMMENTS.as_slice(), comments);
}

#[test]
async fn set_comment_content() {
    const ORIGINAL: &str = "My original comment.";
    const EDIT: &str = "My edit.";

    let original = text::Comment::new(ORIGINAL).unwrap();
    let edit = text::Comment::new(EDIT).unwrap();
    let repo = repo().await;
    let id = repo.add_comment(POST, original.clone()).await.unwrap().id;
    let content = repo.set_comment_content(id, edit).await.unwrap();

    assert_eq!(EDIT, content);

    let comment = repo.get_comment(id).await.unwrap();

    assert_eq!(EDIT, comment.content);

    let id = uuid!("1279acbd-1e9c-4221-8c86-ff4593eb9f94");
    not_found!(repo.set_comment_content(id, original).await, "comment", id);
}
