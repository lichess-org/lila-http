use crate::arena::{ArenaFull, ArenaId};
use moka::future::Cache;

pub struct Repo {
    pub cache: Cache<ArenaId, Option<ArenaFull>>,
}

impl Repo {
    pub fn new() -> Repo {
        Repo {
            cache: Cache::new(1024), // lots of ongoing tournaments (usermade)
        }
    }

    pub async fn get_arena(&self, id: ArenaId) -> Option<ArenaFull> {
        self.cache
            .get_or_insert_with(id.clone(), async move {
                println!("Fetching {}.", id.0);
                Some(ArenaFull {})
            })
            .await
    }
}
