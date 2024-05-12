use fstore::http::Client;
use log::info;
use minty::model::export::Data;
use minty_core::{
    conf::{BucketConfig, Refresh, RepoConfig},
    Repo,
};
use mintyd::{server, Config};
use std::{
    error::Error, fs::File, io::BufReader, path::Path, result, sync::Arc,
};

const CONFIG: &str = "minty-test.toml";

pub type BoxError = Box<dyn Error + Sync + Send + 'static>;
pub type Result<T> = result::Result<T, BoxError>;

fn main() -> Result<()> {
    let mut config = Config::read(Path::new(CONFIG))?;
    config.set_logger()?;
    config.repo.search.refresh = Refresh::WaitFor;

    let _env = minty_core::initialize();

    async_main(config)
}

fn async_main(config: Config) -> Result<()> {
    let body = async move { run(config).await };

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|err| format!("failed to build runtime: {err}"))?
        .block_on(body)
}

async fn run(mut config: Config) -> Result<()> {
    clone_bucket(&mut config.repo.objects).await?;

    let repo = create_repo(&config.repo).await?;
    let result = serve(&config, repo.clone()).await;

    repo.shutdown().await;

    result
}

async fn clone_bucket(config: &mut BucketConfig) -> Result<()> {
    let bucket = &config.bucket;
    let clone = format!("{}-clone", config.bucket);
    let client = Client::new(&config.url);

    info!("Cloning bucket '{bucket}' as '{clone}'");

    match client.get_bucket(&clone).await {
        Ok((_, bucket)) => {
            client.remove_bucket(&bucket.id).await.map_err(|err| {
                format!("failed to remove bucket clone '{clone}': {err}")
            })?
        }
        Err(err) => match err.kind() {
            fstore::ErrorKind::NotFound => (),
            _ => {
                return Err(format!(
                    "failed to get bucket info for {clone}: {err}"
                )
                .into())
            }
        },
    }

    let (original, _) = client.get_bucket(bucket).await.map_err(|err| {
        format!("failed to retrieve original bucket '{bucket}': {err}",)
    })?;

    original
        .clone_as(&clone)
        .await
        .map_err(|err| format!("failed to clone bucket '{bucket}': {err}"))?;

    config.bucket = clone;
    Ok(())
}

async fn create_repo(config: &RepoConfig) -> Result<Arc<Repo>> {
    const BATCH_SIZE: usize = 100;

    let repo = Arc::new(Repo::new(config).await?);

    info!("Initializing database");
    repo.reset().await?;

    info!("Importing test data");
    let data = read_data()?;
    repo.import(&data).await?;

    info!("Building search indices");
    repo.reindex_posts(BATCH_SIZE).await?.1.await??;
    repo.reindex_tags(BATCH_SIZE).await?.1.await??;
    repo.reindex_users(BATCH_SIZE).await?.1.await??;

    Ok(repo)
}

fn read_data() -> Result<Data> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir.join("data.json");
    let file = File::open(&path).map_err(|err| {
        format!("failed to open data file '{}': {err}", path.display())
    })?;
    let reader = BufReader::new(file);
    let data = serde_json::from_reader(reader).map_err(|err| {
        format!("failed to read contents of '{}': {err}", path.display())
    })?;
    Ok(data)
}

async fn serve(config: &Config, repo: Arc<Repo>) -> Result<()> {
    let mut parent = dmon::Parent::default();
    server::serve(&config.http, repo, &mut parent)
        .await
        .map_err(|err| err.to_string())?;

    Ok(())
}
