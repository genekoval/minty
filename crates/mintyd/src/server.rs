mod accept;
mod content;
mod error;
mod query;
mod router;

use crate::{conf::Http, Result};

use accept::Accept;
use axum_unix::shutdown_signal;
use log::{error, info};
use minty_core::Repo;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
struct AppState {
    repo: Arc<Repo>,
}

pub async fn serve(
    config: &Http,
    repo: Arc<Repo>,
    parent: &mut dmon::Parent,
) -> Result {
    info!("minty version {} starting up", minty_core::VERSION);

    repo.prepare().await?;

    let app = router::routes().with_state(AppState { repo });
    let token = CancellationToken::new();

    let mut handles = Vec::new();

    for endpoint in &config.listen {
        let handle =
            axum_unix::serve(endpoint, app.clone(), token.clone(), |_| {
                if let Err(err) = parent.notify() {
                    error!(
                        "Failed to notify parent process of \
                        successful start: {err}"
                    );
                }
            })
            .await;

        match handle {
            Ok(handle) => handles.push(handle),
            Err(err) => error!("{err}"),
        }
    }

    if handles.is_empty() {
        return Err("No servers could be started".into());
    }

    shutdown_signal().await;
    token.cancel();
    info!("Server shutting down");

    for handle in handles {
        if let Err(err) = handle.await {
            error!("Failed to join server task: {err}")
        }
    }

    Ok(())
}
