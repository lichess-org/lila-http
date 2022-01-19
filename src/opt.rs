use std::net::SocketAddr;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Opt {
    /// Binding address. Note that administrative endpoints must be protected
    /// using a reverse proxy.
    #[clap(long, default_value = "127.0.0.1:3000")]
    pub bind: SocketAddr,
    /// Disable access from all origins.
    #[clap(long)]
    pub no_cors: bool,
    /// Base url for the indexer.
    #[clap(long, default_value = "http://l.org")]
    pub lila: String,
    /// Token of https://lichess.org/@/Lilarena to speed up indexing.
    /// The default value is a local dev token, it won't work in production.
    #[clap(long, default_value = "lip_2GB9ilOKexedQCI1xTkP")]
    pub bearer: String,
    #[clap(long, default_value = "mongodb://localhost:27017")]
    pub mongo_url: String,
    #[clap(long, default_value = "lichess")]
    pub mongo_db_name: String,
    #[clap(long, default_value = "redis://localhost")]
    pub redis_url: String,
}
