use std::net::SocketAddr;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Opt {
    /// Binding address.
    #[arg(long, default_value = "127.0.0.1:9001", env = "LILA_HTTP_BIND")]
    pub bind: SocketAddr,
    /// Disable access from all origins, for example if a reverse proxy is
    /// responsible for CORS.
    #[arg(long, env = "LILA_HTTP_NO_CORS")]
    pub no_cors: bool,
    #[command(flatten)]
    pub redis: RedisOpt,
}

#[derive(Parser, Debug)]
pub struct RedisOpt {
    #[arg(long, default_value = "redis://localhost", env = "LILA_HTTP_REDIS_URL")]
    pub redis_url: String,
}
