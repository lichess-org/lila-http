use std::{sync::Arc, time::Duration};

use moka::future::{Cache, CacheBuilder};

use crate::arena::{ArenaFull, ArenaId};

pub struct Repo {
    cache: Cache<ArenaId, Arc<ArenaFull>>,
}

impl Repo {
    pub(crate) fn new() -> Repo {
        Repo {
            cache: CacheBuilder::new(4096) // lots of ongoing tournaments (usermade)
                .time_to_live(Duration::from_secs(15))
                .build(),
        }
    }

    pub fn get(&self, id: ArenaId) -> Option<Arc<ArenaFull>> {
        self.cache.get(&id)
    }

    pub async fn put(&self, full: ArenaFull) {
        self.cache.insert(full.id.clone(), Arc::new(full)).await
    }
}
