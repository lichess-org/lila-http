use crate::arena::{ArenaFull, ArenaId};
use crate::opt::Opt;
use moka::future::{Cache, CacheBuilder};
use std::sync::Arc;
use std::time::Duration;

pub struct Repo {
    pub cache: Cache<ArenaId, ArenaFull>,
    client: reqwest::Client,
    opt: Opt,
}

impl Repo {
    pub fn new(opt: Opt) -> Repo {
        Repo {
            cache: CacheBuilder::new(1024) // lots of ongoing tournaments (usermade)
                .time_to_live(Duration::from_secs(4))
                .build(),
            client: reqwest::Client::new(),
            opt,
        }
    }

    pub async fn get_arena(&self, id: ArenaId) -> Result<ArenaFull, Arc<reqwest::Error>> {
        self.cache
            .get_or_try_insert_with(id.clone(), async move { self.fetch(&id).await })
            .await
    }

    async fn fetch(&self, id: &ArenaId) -> reqwest::Result<ArenaFull> {
        println!("Fetching {}.", id.0);
        let url = format!("{}/tournament/{}/lilarena", self.opt.lila, id.0);
        dbg!(&url);
        let arena: ArenaFull = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.opt.bearer))
            .send()
            .await?
            .json()
            .await?;
        dbg!(&arena);
        Ok(arena)
    }
}
