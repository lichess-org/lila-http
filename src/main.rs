#![forbid(unsafe_code)]

pub mod arena;
pub mod opt;
pub mod redis;
pub mod repo;

use arena::{ArenaId, ClientData, UserName};
use axum::response::{IntoResponse, Response};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    routing::get,
    AddExtensionLayer, Json, Router,
};
use clap::Parser;
use opt::Opt;
use repo::Repo;
use serde::Deserialize;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::new()
            .filter("LILA_HTTP_LOG")
            .write_style("LILA_HTTP_LOG_STYLE"),
    )
    .format_timestamp(None)
    .format_module_path(false)
    .format_target(false)
    .init();

    let opt = dbg!(Opt::parse());

    let repo: &'static Repo = Box::leak(Box::new(Repo::new()));

    tokio::spawn(async move {
        redis::subscribe(opt.redis, repo).await;
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/:id", get(arena))
        .layer(AddExtensionLayer::new(repo));

    let app = if opt.no_cors {
        app
    } else {
        app.layer(
            tower_http::set_header::SetResponseHeaderLayer::if_not_present(
                axum::http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
                axum::http::HeaderValue::from_static("*"),
            ),
        )
    };

    axum::Server::bind(&opt.bind)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Debug, Deserialize)]
struct QueryParams {
    page: Option<usize>,
    me: Option<UserName>,
}

async fn arena(
    Path(id): Path<ArenaId>,
    Query(query): Query<QueryParams>,
    Extension(repo): Extension<&'static Repo>,
) -> Result<Response, StatusCode> {
    let user_id = query.me.map(|n| n.into_id());
    let page = query.page;
    repo.get(id)
        .map(|full| Json(ClientData::new(&full, page, user_id.as_ref())).into_response())
        .ok_or(StatusCode::NOT_FOUND)
}

async fn root() -> &'static str {
    "lila-http"
}
