use std::net::SocketAddr;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Opt {
    /// Binding address.
    #[arg(long, default_value = "127.0.0.1:3000")]
    pub bind: SocketAddr,
    /// Disable access from all origins, for example if a reverse proxy is
    /// responsible for CORS.
    #[arg(long)]
    pub no_cors: bool,
    #[command(flatten)]
    pub redis: RedisOpt,
}

#[derive(Parser, Debug)]
pub struct RedisOpt {
    #[arg(long, default_value = "redis://localhost")]
    pub redis_url: String,
}
