use minty_test::{admin, not_found, objects, posts};

use bytes::Bytes;
use futures::Stream;
use minty::Repo;
use sha2::{Digest, Sha256};
use std::marker::Unpin;
use tokio::{io::AsyncReadExt, test};
use tokio_util::io::StreamReader;
use uuid::{uuid, Uuid};

const IMAGE: Uuid = objects::SAND;
const IMAGE_PREVIEW_HASH: &str =
    "10b3a60302035e5ce4aa08569862a3ca37bca7b9e0075f420c3ed37d6bf1ce73";

const VIDEO: Uuid = objects::BUNNY;
const VIDEO_HASH: &str =
    "7df68aa61121297801587e0318de4eccd30bb96c3a198b45abcc6cffb0cda0f1";
const VIDEO_PREVIEW_HASH: &str =
    "7d1b500cb530df8fadfd54601d1c8bd6998f4800752c25e6839b0a1d3c5cd8d2";
const VIDEO_SIZE: u64 = 673_223_862;

#[test]
async fn get_object() {
    let repo = admin().await;

    let object = repo.get_object(VIDEO).await.unwrap();

    assert_eq!(object.id, VIDEO);
    assert_eq!(object.hash, VIDEO_HASH);
    assert_eq!(object.size, VIDEO_SIZE);
    assert_eq!(object.r#type, "video");
    assert_eq!(object.subtype, "mp4");

    let posts: Vec<_> = object.posts.iter().map(|post| post.id).collect();
    assert!(posts.contains(&posts::BUNNY));

    let id = uuid!("5909db6d-2ced-47fc-8824-781f0e68cf8f");
    not_found!(repo.get_object(id).await, "object", id);
}

#[test]
async fn get_object_data() {
    let repo = admin().await;

    let id = uuid!("f0379789-9af1-4f73-88ae-9edf6cf6751f");
    not_found!(
        repo.get_object_data(id).await.map(|(summary, _)| summary),
        "object",
        id
    );

    let (summary, stream) = repo.get_object_data(VIDEO).await.unwrap();

    assert_eq!(summary.media_type, "video/mp4");
    assert_eq!(summary.size, VIDEO_SIZE);

    let hash = sha256sum(stream).await;

    assert_eq!(hash, VIDEO_HASH);
}

#[test]
async fn image_preview() {
    test_preview(IMAGE, IMAGE_PREVIEW_HASH).await;
}

#[test]
async fn video_preview() {
    test_preview(VIDEO, VIDEO_PREVIEW_HASH).await;
}

async fn test_preview(object: Uuid, expected_hash: &str) {
    let repo = admin().await;
    let preview = repo.get_object(object).await.unwrap().preview_id.unwrap();
    let (_, stream) = repo.get_object_data(preview).await.unwrap();
    let hash = sha256sum(stream).await;

    assert_eq!(expected_hash, hash);
}

async fn sha256sum(
    stream: impl Stream<Item = std::io::Result<Bytes>> + Unpin,
) -> String {
    let mut hasher = Sha256::new();
    let mut reader = StreamReader::new(stream);
    let mut buf = [0u8; 8192];

    loop {
        let n = reader.read(&mut buf).await.unwrap();
        if n == 0 {
            break;
        }

        hasher.update(&buf[..n]);
    }

    let hash = hasher.finalize();

    let mut buf = [0u8; 64];
    let hash = base16ct::lower::encode_str(&hash, &mut buf).unwrap();

    String::from(hash)
}
