use axum::{
    // extract::{Extension, Path, Query},
    extract::{Extension, Path},
    routing::get,
    AddExtensionLayer,
    // http::StatusCode,
    // response::IntoResponse,
    Router, // Json,
};
// use serde::{Deserialize, Serialize};
use clap::Parser;
use std::net::SocketAddr;

#[derive(Parser, Debug, Clone)]
struct Opt {
    /// Binding address. Note that administrative endpoints must be protected
    /// using a reverse proxy.
    #[clap(long, default_value = "127.0.0.1:3000")]
    bind: SocketAddr,
    /// Disable access from all origins.
    #[clap(long)]
    nocors: bool,
    /// Base url for the indexer.
    #[clap(long = "lila", default_value = "http://l.org")]
    lila: String,
}

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

    let app = Router::new()
        .route("/", get(root))
        .route("/:id", get(arena))
        .layer(AddExtensionLayer::new(opt.clone()));

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

// #[serde_as]
// #[derive(Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
// pub struct ArenaId(str);

// async fn arena(Path(ArenaId(id))) -> Result<str> {
//     id
// }

async fn arena(Path(arena_id): Path<String>, Extension(opt): Extension<Opt>) -> String {
    // reqwest::get("http://httpbin.org/ip").await?.text().await
    // reqwest::get("http://httpbin.org/ip").await.unwrap().text().await.unwrap()

    let body = fetch(&arena_id, &opt).await;
    let res = match body {
        Err(err) => format!("Oh no, an error message with code 200! {}", err),
        Ok(b) => b,
    };
    // let res = body.map_err(|err| format!("Oh no, an error message with code 200! {}", err));
    res
}

// ok. goal is to return some sort of Result<string>
// where a response status != 200 returns an error
async fn fetch(id: &str, opt: &Opt) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/tournament/{}", opt.lila, id);
    dbg!(&url);
    let res = client
        .get(url)
        .header("Accept", "application/vnd.lichess.v5+json")
        .send()
        .await;
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
