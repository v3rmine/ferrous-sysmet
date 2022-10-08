use axum::{
    extract::{Extension, Query},
    routing::get,
    Router, Server,
};
pub use eyre::{Error, Result};
use include_dir::{include_dir, Dir};
use log::{debug, info, trace, tracing};
use maud::{html, Markup};
use metrics::Database;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{collections::HashMap, net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};
use tokio::{sync::RwLock, time::Instant};
use typed_builder::TypedBuilder;

mod components;
pub use components::*;
mod macros;

pub(crate) const SOURCE_URL: &str = "https://github.com/joxcat/sysmet";
pub(crate) const WEBSITE_TITLE: &str = "Ferrous System Metrics";

pub(crate) const CSS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/css/exports");
pub(crate) static CSS_HASHES: Lazy<HashMap<String, (PathBuf, String)>> =
    generate_hashes!(CSS_HASHES, CSS_DIR);
static_files_server!(css_assets, CSS_DIR, CSS_HASHES, "text/css");

const ACTUALIZATION_INTERVAL: Duration = Duration::from_secs(120);

#[derive(Debug, TypedBuilder)]
struct ChartData {
    pub last_updated_time: Instant,
    pub metrics: Database,
}

#[tracing::instrument]
pub async fn run_server(addr: SocketAddr, database: &str) -> Result<()> {
    let chart_data = RwLock::new(
        ChartData::builder()
            .last_updated_time(Instant::now())
            .metrics(Database::default())
            .build(),
    );
    let shared_chart_data = Arc::new(chart_data);

    let (db_tx, mut db_rx) = tokio::sync::oneshot::channel::<()>();
    let (server_tx, server_rx) = tokio::sync::oneshot::channel::<()>();
    let handle = {
        let shared_chart_data = shared_chart_data.clone();
        let database = database.to_string();

        tokio::spawn(async move {
            debug!("Spawned actualization task");
            let mut interval = tokio::time::interval(ACTUALIZATION_INTERVAL);

            loop {
                let interval = interval.tick();
                tokio::pin!(interval);

                tokio::select! {
                    _ = &mut interval => {
                        if let Ok(database) = Database::from_file(&database) {
                            let mut chart_data = shared_chart_data.write().await;
                            chart_data.metrics = database;
                            chart_data.last_updated_time = Instant::now();
                        }
                    }
                    _ = &mut db_rx => {
                        break;
                    }
                }

                trace!("Looped through actualization process");
            }

            debug!("Finished actualization task");
        })
    };

    {
        tokio::spawn(async move {
            debug!("Spawned Ctrl-C handler task");
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install CTRL+C signal handler");
            trace!("Catched Ctrl-C");

            db_tx
                .send(())
                .expect("Failed to send stop signal to the actualization task");
            trace!("Waiting for the actualization task to end");
            handle
                .await
                .expect("Error while waiting for the end of the actualization task");
            trace!("Actualization task ended");
            server_tx
                .send(())
                .expect("Failed to send stop signal to the server task");

            debug!("Finished Ctrl-C gracefull shutdown");
        });
    }

    let app = Router::new()
        .route("/", get(home))
        .route("/css/:path", get(css_assets))
        .layer(Extension(shared_chart_data));

    info!("Listening on {}", addr);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async {
            server_rx.await.ok();
            trace!("Received server stop signal");
        })
        .await?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct HomeQuery {
    t: Option<String>,
    refresh: Option<String>,
}

const CPU_USAGE_TITLE: &str = "CPU Usage";
// const RAM_USAGE_TITLE: &str = "RAM Usage";
// const LOAD_AVERAGE_TITLE: &str = "Load Average";
// const NETWORK_TITLE: &str = "Network";
// const DISKS_TITLE: &str = "Disks";

#[tracing::instrument]
async fn home(
    time_from_now: Query<HomeQuery>,
    Extension(chart_data): Extension<Arc<RwLock<ChartData>>>,
) -> Markup {
    let time_from_now = time_from_now.0;
    let _time = time_from_now
        .t
        .clone()
        .and_then(|ref t| humantime::parse_duration(t).ok());
    let refresh = time_from_now.refresh.is_some() && time_from_now.refresh.clone().unwrap() == "on";

    let chart_data = {
        let data = chart_data.read().await;
        data.metrics.clone()
    };

    let snapshots_len = chart_data.snapshots.len();

    let mut cpus_usages = Vec::with_capacity(snapshots_len);
    for (snap, timestamp) in chart_data.get_cpu_usages() {
        cpus_usages.push((snap, timestamp.timestamp(), ()) as ChartValue<_>);
    }

    let chart_sections = chart_data
        .snapshots
        .into_iter()
        .fold(
            vec![(CPU_USAGE_TITLE, cpus_usages)]
                .into_iter()
                .collect::<HashMap<&str, Vec<ChartValue<()>>>>(),
            |collections, snapshot| {
                let _timestamp = snapshot.time.timestamp();

                collections
            },
        )
        .into_iter()
        .map(|(title, values)| {
            (
                title,
                match title {
                    CPU_USAGE_TITLE => ChartContext::builder()
                        .collections(vec![("#e00", values)])
                        .build(),
                    _ => unreachable!(),
                },
            )
        })
        .collect::<Vec<_>>();

    Base(
        BaseContext::builder().refresh_every_minute(refresh).build(),
        html! {
            section {
                h1 { "sysmet faster" }
                form {
                    div {
                        label {
                            span { "Time range:" }
                            input name="t" value=(time_from_now.t.unwrap_or_else(|| "3h0m0s".to_string()));
                            span { "ago to now." }
                        }
                        label {
                            @if refresh {
                                input type="checkbox" name="refresh" checked;
                            } @else {
                                input type="checkbox" name="refresh";
                            }
                            span { "Auto-refresh every minute" }
                        }
                    }
                    input type="submit" { "Change" }
                }
            }
            section {
                @for (title, context) in chart_sections {
                    section {
                        h2 { (title) }
                        (Chart(context))
                    }
                }
            }
            section {
                a href=(SOURCE_URL) referer="none" target="_blank" { "Source code" }
                span { " - Licensed under the AGPL v3.0." }
            }
        },
    )
}
