use minty_test::{new_user, next_user, repo, sign_up_info, users, ResultExt};

use minty::{
    text::{Description, Email, Name, Password},
    Login, Pagination, ProfileQuery, Repo, Url,
};
use tokio::test;

#[test]
async fn sign_up() {
    let info = sign_up_info("sign-up-test");
    let repo = minty_test::sign_up(&info).await;
    let user = repo.get_authenticated_user().await.unwrap();

    assert_eq!(user.email, info.email.as_ref());
    assert_eq!(user.profile.name, info.username.as_ref());
    assert!(!user.admin);

    repo.delete_user().await.unwrap();
    repo.get_authenticated_user().await.expect_unauthenticated();
    self::repo()
        .get_authenticated_user()
        .await
        .expect_unauthenticated();
}

#[test]
async fn add_alias() {
    const NAME: &str = "add-alias";
    const ALIAS: &str = "User Alias";

    let alias = Name::new(ALIAS).unwrap();
    let repo = new_user(NAME).await;
    let user = repo.add_user_alias(alias.clone()).await.unwrap();

    assert_eq!(user.name, NAME);
    assert_eq!(user.aliases.len(), 1);
    assert_eq!(user.aliases.first().unwrap(), ALIAS);

    let user = repo.get_authenticated_user().await.unwrap();

    assert_eq!(user.profile.name, NAME);
    assert_eq!(user.profile.aliases.len(), 1);
    assert_eq!(user.profile.aliases.first().unwrap(), ALIAS);

    repo.delete_user().await.unwrap();
    self::repo()
        .get_authenticated_user()
        .await
        .expect_unauthenticated();
}

#[test]
async fn add_source() {
    const SOURCE: &str = "https://example.com/hello";

    let url = Url::parse(SOURCE).unwrap();
    let repo = next_user().await;
    let source = repo.add_user_source(&url).await.unwrap();

    assert_eq!(url, source.url);
    assert!(source.icon.is_none());

    let user = repo.get_authenticated_user().await.unwrap();

    assert_eq!(user.profile.sources.len(), 1);

    let result = user.profile.sources.first().unwrap();

    assert_eq!(source.id, result.id);
    assert_eq!(source.url, result.url);
    assert_eq!(source.icon, result.icon);

    repo.delete_user().await.unwrap();
    self::repo()
        .add_user_source(&url)
        .await
        .expect_unauthenticated();
}

#[test]
async fn delete_alias() {
    const NAME: &str = "delete-alias-name";
    const ALIAS: &str = "Delete Me";

    let alias = Name::new(ALIAS).unwrap();
    let repo = new_user(NAME).await;
    repo.add_user_alias(alias).await.unwrap();

    for _ in 0..2 {
        let user = repo.delete_user_alias(ALIAS).await.unwrap();
        assert_eq!(NAME, user.name);
        assert!(user.aliases.is_empty());

        let user = repo.get_authenticated_user().await.unwrap();
        assert_eq!(NAME, user.profile.name);
        assert!(user.profile.aliases.is_empty());
    }

    repo.delete_user().await.unwrap();
    self::repo()
        .delete_user_alias(ALIAS)
        .await
        .expect_unauthenticated();
}

#[test]
async fn delete_source() {
    let repo = next_user().await;
    let url = Url::parse("https://example.com/hello").unwrap();
    let source = repo.add_user_source(&url).await.unwrap();

    repo.delete_user_source(source.id).await.unwrap();

    let user = repo.get_authenticated_user().await.unwrap();

    assert!(user.profile.sources.is_empty());

    repo.delete_user_source(source.id).await.expect_not_found();

    repo.delete_user().await.unwrap();
    self::repo()
        .delete_user_source(source.id)
        .await
        .expect_unauthenticated();
}

#[test]
async fn delete_sources() {
    const HOST: &str = "example.com";

    let repo = next_user().await;

    for path in ["hello/world", "foo/bar"] {
        let url = format!("https://{HOST}");
        let mut url = Url::parse(&url).unwrap();
        url.set_path(path);
        repo.add_user_source(&url).await.unwrap();
    }

    repo.delete_user_sources(&[HOST.to_owned()]).await.unwrap();

    let user = repo.get_authenticated_user().await.unwrap();

    assert!(
        user.profile.sources.is_empty(),
        "user.sources = {:?}",
        user.profile.sources
    );

    repo.delete_user().await.unwrap();
    self::repo()
        .delete_user_sources(&[HOST.to_owned()])
        .await
        .expect_unauthenticated();
}

#[test]
async fn get_user() {
    let user = repo().get_user(users::MINTY).await.unwrap();

    assert_eq!(user.id, users::MINTY);
    assert_eq!(user.email, "minty@example.com");
    assert_eq!(user.profile.name, "minty");
}

#[test]
async fn get_users() {
    let repo = next_user().await;
    let user = repo.get_authenticated_user().await.unwrap().id;

    let query = ProfileQuery {
        pagination: Pagination {
            from: 0,
            size: 1_000,
        },
        name: "minty".into(),
        exclude: Default::default(),
    };

    let result = repo.get_users(&query).await.unwrap();

    assert!(result.total >= 1, "result.total = {}", result.total);

    let hits: Vec<_> = result.hits.iter().map(|hit| hit.id).collect();

    assert!(hits.contains(&user));

    repo.delete_user().await.unwrap();
}

#[test]
async fn set_description() {
    const DESCRIPTION: &str = "A description for a user.";

    let description = Description::new(DESCRIPTION).unwrap();
    let repo = next_user().await;
    let result = repo
        .set_user_description(description.clone())
        .await
        .unwrap();
    assert_eq!(result, DESCRIPTION);

    let user = repo.get_authenticated_user().await.unwrap();

    assert_eq!(user.profile.description, DESCRIPTION);

    repo.delete_user().await.unwrap();
    self::repo()
        .set_user_description(description)
        .await
        .expect_unauthenticated();
}

#[test]
async fn set_email() {
    const EMAIL: &str = "new@example.com";

    let email = Email::new(EMAIL).unwrap();
    let repo = next_user().await;

    repo.set_user_email(email.clone()).await.unwrap();

    let user = repo.get_authenticated_user().await.unwrap();

    assert_eq!(user.email, EMAIL);

    repo.delete_user().await.unwrap();
    self::repo()
        .set_user_email(email)
        .await
        .expect_unauthenticated();
}

#[test]
async fn set_name() {
    const NAME: &str = "set-name";
    const ALIAS: &str = "User Alias";

    let name = Name::new(NAME).unwrap();
    let alias = Name::new(ALIAS).unwrap();
    let repo = new_user(NAME).await;
    let user = repo.set_user_name(alias).await.unwrap();

    assert_eq!(user.name, ALIAS);
    assert!(user.aliases.is_empty());

    repo.add_user_alias(name.clone()).await.unwrap();
    let user = repo.set_user_name(name.clone()).await.unwrap();

    assert_eq!(user.name, NAME);
    assert_eq!(user.aliases.len(), 1);
    assert_eq!(user.aliases.first().unwrap(), ALIAS);

    let user = repo.get_authenticated_user().await.unwrap();

    assert_eq!(user.profile.name, NAME);
    assert_eq!(user.profile.aliases.len(), 1);
    assert_eq!(user.profile.aliases.first().unwrap(), ALIAS);

    repo.delete_user().await.unwrap();
    self::repo()
        .set_user_name(name)
        .await
        .expect_unauthenticated();
}

#[test]
async fn set_password() {
    const PASSWORD: &str = "my.super.secret.password";

    let info = sign_up_info("set-password");

    let password = Password::new(PASSWORD).unwrap();
    let repo = minty_test::sign_up(&info).await;
    let id = repo.get_authenticated_user().await.unwrap().id;

    repo.set_user_password(password.clone()).await.unwrap();
    repo.sign_out().await.unwrap();

    let login = Login {
        email: info.email.to_string(),
        password: PASSWORD.to_owned(),
    };

    repo.authenticate(&login).await.unwrap();

    let user = repo.get_authenticated_user().await.unwrap();

    assert_eq!(user.id, id);

    repo.delete_user().await.unwrap();
    self::repo()
        .set_user_password(password)
        .await
        .expect_unauthenticated();
}
