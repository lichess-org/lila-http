use std::net::SocketAddr;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Opt {
    /// Binding address.
    #[clap(long, default_value = "127.0.0.1:3000")]
    pub bind: SocketAddr,
    /// Disable access from all origins, for example if a reverse proxy is
    /// responsible for CORS.
    #[clap(long)]
    pub no_cors: bool,
    #[clap(flatten)]
    pub redis: RedisOpt,
}

#[derive(Parser, Debug)]
pub struct RedisOpt {
    #[clap(long, default_value = "redis://localhost")]
    pub redis_url: String,
}
