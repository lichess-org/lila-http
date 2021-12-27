#[derive(Parser, Clone)]
pub struct ArenaOpt {
    /// Base url for the indexer.
    #[clap(long = "lila", default_value = "https://lichess.org")]
    lila: String,
}

pub struct Lila {
    client: reqwest::Client,
    opt: ArenaOpt,
}

impl Lila {
    pub fn new(opt: IndexerOpt) -> Lila {
        Lila {
            client: reqwest::Client::builder().build().expect("reqwest client"),
            opt,
        }
    }
}
