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
    #[clap(long = "bearer", default_value = "lip_2GB9ilOKexedQCI1xTkP")]
    pub bearer: String,
}
