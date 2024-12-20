#![forbid(unsafe_code)]

pub mod arena;
pub mod opt;
pub mod redis;
pub mod repo;

use std::sync::atomic::{AtomicU64, Ordering};

use arena::{ArenaId, ClientData, UserName};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Router,
};
use axum_extra::response::ErasedJson;
use clap::Parser;
use listenfd::ListenFd;
use opt::Opt;
use repo::Repo;
use serde::Deserialize;
use tikv_jemallocator::Jemalloc;
use tokio::net::TcpListener;

use crate::redis::RedisStats;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[derive(Default)]
struct HttpStats {
    hit: AtomicU64,
    miss: AtomicU64,
}

#[derive(Clone)]
struct AppState {
    redis_stats: &'static RedisStats,
    http_stats: &'static HttpStats,
    repo: &'static Repo,
}

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

    let state = AppState {
        redis_stats: Box::leak(Box::default()),
        http_stats: Box::leak(Box::default()),
        repo: Box::leak(Box::new(Repo::new())),
    };

    tokio::spawn(async move {
        redis::subscribe(opt.redis, state.repo, state.redis_stats).await;
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/tournament/{id}", get(arena))
        .with_state(state);

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

    let listener = match ListenFd::from_env()
        .take_tcp_listener(0)
        .expect("tcp listener")
    {
        Some(std_listener) => {
            std_listener.set_nonblocking(true).expect("set nonblocking");
            TcpListener::from_std(std_listener).expect("listener")
        }
        None => TcpListener::bind(&opt.bind).await.expect("bind"),
    };

    axum::serve(listener, app).await.expect("serve");
}

#[derive(Debug, Deserialize)]
struct QueryParams {
    page: Option<usize>,
    me: Option<UserName>,
}

async fn arena(
    State(state): State<AppState>,
    Path(id): Path<ArenaId>,
    Query(query): Query<QueryParams>,
) -> Result<ErasedJson, StatusCode> {
    let user_id = query.me.map(UserName::into_id);
    let page = query.page;
    state
        .repo
        .get(id)
        .await
        .map(|full| {
            state.http_stats.hit.fetch_add(1, Ordering::Relaxed);
            ErasedJson::new(ClientData::new(&full, page, user_id.as_ref()))
        })
        .ok_or_else(|| {
            state.http_stats.miss.fetch_add(1, Ordering::Relaxed);
            StatusCode::NOT_FOUND
        })
}

async fn root(State(state): State<AppState>) -> String {
    let http_hit = state.http_stats.hit.load(Ordering::Relaxed);
    let http_miss = state.http_stats.miss.load(Ordering::Relaxed);
    let redis_messages = state.redis_stats.messages.load(Ordering::Relaxed);
    let repo_count = state.repo.entry_count();
    format!(
        "lila_http {}",
        [
            // HttpStats
            format!("http_hit={http_hit}u"),
            format!("http_miss={http_miss}u"),
            // RedisStats
            format!("redis_messages={redis_messages}u"),
            // Repo
            format!("repo_count={repo_count}u"),
        ]
        .join(",")
    )
}
