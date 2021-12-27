use axum::{
    // extract::{Extension, Path, Query},
    extract::Path,
    routing::get,
    // http::StatusCode,
    // response::IntoResponse,
    Router, // Json,
};
// use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

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
    log::debug!("Starting HTTP server");

    let app = Router::new()
        .route("/", get(root))
        .route("/:id", get(arena));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// #[serde_as]
// #[derive(Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
// pub struct ArenaId(str);

// async fn arena(Path(ArenaId(id))) -> Result<str> {
//     id
// }

async fn arena(Path(arena_id): Path<String>) -> String {
    // reqwest::get("http://httpbin.org/ip").await?.text().await
    // reqwest::get("http://httpbin.org/ip").await.unwrap().text().await.unwrap()

    let body = fetch(&arena_id).await;
    let res = match body {
        Err(err) => format!("Oh no, an error message with code 200! {}", err),
        Ok(b) => format!("{}: {}", arena_id, b),
    };
    res
}

// ok. goal is to return some sort of Result<string>
// where a response status != 200 returns an error
async fn fetch(id: &str) -> Result<String, String> {
    let res = reqwest::get(format!("http://l.org/tournament/{}", id)).await;
    let res = match res {
        Err(e) => return Err(format!("Couldn't fetch: {}", e)),
        Ok(r) => r,
    };
    if res.status() == 200 {
        res.text()
            .await
            .map_err(|e| format!("Can't get response text body (?!): {}", e))
    } else {
        Err(format!("status: {}", res.status()))
    }
}

async fn root() -> &'static str {
    "lilarena"
}
