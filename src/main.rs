pub mod arena;
pub mod http;
pub mod opt;
pub mod redis;
pub mod repo;

use std::sync::Arc;

use arena::{ArenaId, ClientData, UserName};
use axum::{
    extract::{Extension, Path, Query},
    routing::get,
    AddExtensionLayer, Json, Router,
};
use clap::Parser;
use opt::Opt;
use repo::Repo;
use serde::Deserialize;

use crate::http::{not_found, HttpResponseError};

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

    let opt = Opt::parse();
    dbg!(&opt);

    let repo = Arc::new(Repo::new());
    redis::subscribe(opt.clone(), Arc::clone(&repo)).unwrap();

    let app = Router::new()
        .route("/", get(root))
        .route("/:id", get(arena))
        .layer(AddExtensionLayer::new(opt.clone()))
        .layer(AddExtensionLayer::new(repo));

    let app = if opt.nocors {
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct QueryParams {
    page: Option<usize>,
    me: Option<UserName>,
}

async fn arena(
    Path(id): Path<ArenaId>,
    Query(query): Query<QueryParams>,
    Extension(repo): Extension<Arc<Repo>>,
) -> Result<Json<ClientData>, HttpResponseError> {
    let user_id = query.me.map(|n| n.to_id());
    let page = query.page;
    repo.get(id)
        .map(|full| Json(ClientData::new(full, page, user_id)))
        .ok_or_else(not_found)
}

async fn root() -> &'static str {
    "lila-http"
}
