mod error;
mod router;

use crate::{conf::Http, Result};

use log::{error, info};
use minty_core::Repo;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    repo: Arc<Repo>,
}

pub async fn serve(
    config: &Http,
    repo: Arc<Repo>,
    parent: &mut dmon::Parent,
) -> Result {
    info!("minty version {} starting up", repo.about().version.number);

    repo.prepare().await?;

    let app = router::routes().with_state(AppState { repo });

    axum_unix::serve(&config.listen, app, |_| {
        if let Err(err) = parent.notify() {
            error!(
                "Failed to notify parent process of successful start: {err}"
            );
        }
    })
    .await?;

    info!("Server shutting down");
    Ok(())
}
