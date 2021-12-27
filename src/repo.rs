use crate::arena::{ArenaFull, ArenaId};
use moka::future::{Cache, CacheBuilder};
use std::sync::Arc;
use std::time::Duration;

pub struct Repo {
    pub cache: Cache<ArenaId, ArenaFull>,
    client: reqwest::Client,
    lila: String,
}

impl Repo {
    pub fn new(lila: String) -> Repo {
        Repo {
            cache: CacheBuilder::new(1024) // lots of ongoing tournaments (usermade)
                .time_to_live(Duration::from_secs(4))
                .build(),
            client: reqwest::Client::new(),
            lila,
        }
    }

    pub async fn get_arena(&self, id: ArenaId) -> Result<ArenaFull, Arc<reqwest::Error>> {
        self.cache
            .get_or_try_insert_with(id.clone(), async move { self.fetch(&id).await })
            .await
    }

    async fn fetch(&self, id: &ArenaId) -> reqwest::Result<ArenaFull> {
        println!("Fetching {}.", id.0);
        let url = format!("{}/tournament/{}/lilarena", self.lila, id.0);
        dbg!(&url);
        let arena: ArenaFull = self.client.get(url).send().await?.json().await?;
        dbg!(&arena);
        Ok(arena)
    }
}
