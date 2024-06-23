use minty_test::{repo, users, ResultExt};

use function_name::named;
use minty::{
    http,
    text::{Description, Email, Name, Password},
    Login, Pagination, ProfileQuery, Repo, SignUp, Url,
};
use tokio::test;

macro_rules! new_user {
    () => {
        sign_up_and(function_name!()).await
    };
}

fn info(name: &str) -> SignUp {
    let email = format!("{name}@example.com");

    SignUp {
        username: Name::new(name).unwrap(),
        email: Email::new(&email).unwrap(),
        password: Password::new("password").unwrap(),
    }
}

async fn sign_up_and(name: &str) -> http::Repo {
    let info = info(name);
    repo().sign_up_and(&info, None).await.unwrap()
}

#[test]
#[named]
async fn sign_up() {
    let info = info(function_name!());
    let repo = repo().sign_up_and(&info, None).await.unwrap();
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
#[named]
async fn add_alias() {
    const NAME: &str = function_name!();
    const ALIAS: &str = "User Alias";

    let alias = Name::new(ALIAS).unwrap();
    let repo = sign_up_and(NAME).await;
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
#[named]
async fn add_source() {
    const SOURCE: &str = "https://example.com/hello";

    let url = Url::parse(SOURCE).unwrap();
    let repo = new_user!();
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
#[named]
async fn delete_source() {
    let repo = new_user!();
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
#[named]
async fn delete_sources() {
    const HOST: &str = "example.com";

    let repo = new_user!();

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
    let repo = sign_up_and("get_users").await;
    let user = repo.get_authenticated_user().await.unwrap().id;

    let query = ProfileQuery {
        pagination: Pagination {
            from: 0,
            size: 1_000,
        },
        name: "get".into(),
        exclude: Default::default(),
    };

    let result = repo.get_users(&query).await.unwrap();

    assert!(result.total >= 1, "result.total = {}", result.total);

    let hits: Vec<_> = result.hits.iter().map(|hit| hit.id).collect();

    assert!(hits.contains(&user));

    repo.delete_user().await.unwrap();
}

#[test]
#[named]
async fn set_description() {
    const DESCRIPTION: &str = "A description for a user.";

    let description = Description::new(DESCRIPTION).unwrap();
    let repo = new_user!();
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
#[named]
async fn set_email() {
    const EMAIL: &str = "new@example.com";

    let email = Email::new(EMAIL).unwrap();
    let repo = new_user!();

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
#[named]
async fn set_name() {
    const NAME: &str = function_name!();
    const ALIAS: &str = "User Alias";

    let name = Name::new(NAME).unwrap();
    let alias = Name::new(ALIAS).unwrap();
    let repo = new_user!();
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
#[named]
async fn set_password() {
    const PASSWORD: &str = "my.super.secret.password";

    let password = Password::new(PASSWORD).unwrap();
    let mut repo = new_user!();
    let id = repo.get_authenticated_user().await.unwrap().id;

    repo.set_user_password(password.clone()).await.unwrap();
    repo.sign_out().await.unwrap();

    let login = Login {
        email: format!("{}@example.com", function_name!()),
        password: PASSWORD.to_owned(),
    };

    repo = self::repo().with_user(&login).await.unwrap();

    let user = repo.get_authenticated_user().await.unwrap();

    assert_eq!(user.id, id);

    repo.delete_user().await.unwrap();
    self::repo()
        .set_user_password(password)
        .await
        .expect_unauthenticated();
}
