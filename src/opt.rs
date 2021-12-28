use clap::Parser;
use std::net::SocketAddr;

#[derive(Parser, Debug, Clone)]
pub struct Opt {
    /// Binding address. Note that administrative endpoints must be protected
    /// using a reverse proxy.
    #[clap(long, default_value = "127.0.0.1:3000")]
    pub bind: SocketAddr,
    /// Disable access from all origins.
    #[clap(long)]
    pub nocors: bool,
    /// Base url for the indexer.
    #[clap(long = "lila", default_value = "http://l.org")]
    pub lila: String,
    /// Token of https://lichess.org/@/Lilarena to speed up indexing.
    /// The default value is a local dev token, it won't work in production.
    #[clap(long = "bearer", default_value = "lip_2GB9ilOKexedQCI1xTkP")]
    pub bearer: String,
    #[clap(long = "mongo_url", default_value = "mongodb://localhost:27017")]
    pub mongo_url: String,
    #[clap(long = "mongo_db_name", default_value = "lichess")]
    pub mongo_db_name: String,
    #[clap(long = "redis_url", default_value = "redis://localhost")]
    pub redis_url: String,
}
