#![forbid(unsafe_code)]

pub mod arena;
pub mod opt;
pub mod redis;
pub mod repo;

use std::sync::atomic::{AtomicU64, Ordering};

use actix_web::{App, HttpResponse, HttpServer, dev::Service, http::header, web};
use arena::{ArenaId, ClientData, UserName};
use clap::Parser;
use listenfd::ListenFd;
use opt::Opt;
use repo::Repo;
use serde::Deserialize;
use tikv_jemallocator::Jemalloc;

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

    let state = web::Data::new(state);
    let no_cors = opt.no_cors;
    let server = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/", web::get().to(root))
            .route("/tournament/{id}", web::get().to(arena))
            .wrap_fn(move |request, service| {
                let response = service.call(request);
                async move {
                    let mut response = response.await?;
                    if !no_cors
                        && !response
                            .headers()
                            .contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                    {
                        response.headers_mut().insert(
                            header::ACCESS_CONTROL_ALLOW_ORIGIN,
                            header::HeaderValue::from_static("*"),
                        );
                    }
                    Ok(response)
                }
            })
    });

    let mut fds = ListenFd::from_env();
    if let Ok(Some(uds)) = fds.take_unix_listener(0) {
        uds.set_nonblocking(true).expect("set nonblocking");
        server
            .listen_uds(uds)
            .expect("listen")
            .run()
            .await
            .expect("serve");
    } else if let Ok(Some(tcp)) = fds.take_tcp_listener(0) {
        tcp.set_nonblocking(true).expect("set nonblocking");
        server
            .listen(tcp)
            .expect("listen")
            .run()
            .await
            .expect("serve");
    } else {
        server
            .bind(opt.bind)
            .expect("bind")
            .run()
            .await
            .expect("serve");
    }
}

#[derive(Debug, Deserialize)]
struct QueryParams {
    page: Option<usize>,
    me: Option<UserName>,
}

async fn arena(
    state: web::Data<AppState>,
    id: web::Path<ArenaId>,
    query: web::Query<QueryParams>,
) -> HttpResponse {
    let QueryParams { page, me } = query.into_inner();
    let user_id = me.map(UserName::into_id);
    match state.repo.get(id.into_inner()).await {
        Some(full) => {
            state.http_stats.hit.fetch_add(1, Ordering::Relaxed);
            match serde_json::to_vec(&ClientData::new(&full, page, user_id.as_ref())) {
                Ok(json) => HttpResponse::Ok()
                    .content_type("application/json")
                    .body(json),
                Err(error) => {
                    log::error!("failed to serialize tournament response: {error}");
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
        None => {
            state.http_stats.miss.fetch_add(1, Ordering::Relaxed);
            HttpResponse::NotFound().finish()
        }
    }
}

async fn root(state: web::Data<AppState>) -> String {
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
