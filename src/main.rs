pub mod arena;
pub mod opt;
pub mod repo;

use arena::{ArenaFull, ArenaId};
use axum::{
    extract::{Extension, Path},
    routing::get,
    AddExtensionLayer, Router,
};
use clap::Parser;
use opt::Opt;
use repo::Repo;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::new()
            .filter("ARENA_LOG")
            .write_style("ARENA_LOG_STYLE"),
    )
    .format_timestamp(None)
    .format_module_path(false)
    .format_target(false)
    .init();

    let opt = Opt::parse();
    dbg!(&opt);

    let repo = Arc::new(Repo::new(opt.clone()));

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

async fn arena(
    Path(id): Path<ArenaId>,
    Extension(opt): Extension<Opt>,
    Extension(repo): Extension<Arc<Repo>>,
) -> ArenaFull {
    let arena = repo.get_arena(id).await;
    match arena {
        Err(err) => format!("Oh no, an error message with code 200! {}", err),
        Ok(b) => b,
    }
}

async fn root() -> &'static str {
    "lilarena"
}
