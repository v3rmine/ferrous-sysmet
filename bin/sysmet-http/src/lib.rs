use axum::{routing::get, Router, Server};
pub use eyre::{Error, Result};
use include_dir::{include_dir, Dir};
use log::{info, tracing};
use maud::{html, Markup};

use std::net::SocketAddr;

mod components;
pub use components::*;
mod macros;

static CSS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/css/exports");
static_files_server!(css_assets, CSS_DIR, "text/css");

#[tracing::instrument]
pub async fn run_server(addr: SocketAddr) -> Result<()> {
    let app = Router::new()
        .route("/", get(home))
        .route("/css/:path", get(css_assets));

    info!("Listening on {}", addr);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

#[tracing::instrument]
async fn home() -> Markup {
    Base(html! {
        h1 { "Hello World" }
    })
}
