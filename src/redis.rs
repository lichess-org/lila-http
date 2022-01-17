use crate::arena::{
    ArenaFull, ArenaId, ArenaShared, FullRanking, GameId, Player, Rank, Sheet, SheetScores, UserId,
    UserName,
};
use crate::repo::Repo;
use futures::stream::StreamExt;
use log::error;
use redis::RedisError;
use serde::Deserialize;
use serde_json::Error as SerdeJsonError;
use serde_json::Value as JsValue;
use serde_with::serde_as;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("[REDIS] Error getting payload: {0}")]
    RedisError(#[from] RedisError),
    #[error("[SERDE] Error parsing JSON: {0}")]
    SerdeJsonError(#[from] SerdeJsonError),
}

pub fn parse_message(msg: &redis::Msg) -> Result<ArenaFullRedis, Error> {
    let str = &msg.get_payload::<String>()?;
    let res = serde_json::from_str(str)?;
    Ok(res)
}

pub fn subscribe(opt: crate::opt::Opt, repo: Arc<Repo>) -> Result<(), Error> {
    let _ = tokio::spawn(async move {
        let client = redis::Client::open(opt.redis_url).unwrap();
        let subscribe_con = client.get_tokio_connection().await.unwrap();
        let mut pubsub = subscribe_con.into_pubsub();
        pubsub.subscribe("http-out").await.unwrap();
        let mut stream = pubsub.on_message();
        while let Some(msg) = stream.next().await {
            match parse_message(&msg) {
                Ok(full) => repo.put(full.expand()).await,
                Err(msg) => error!("{:?}", dbg!(msg)),
            }
        }
    });
    Ok(())
}

#[derive(Deserialize, Clone, Debug)]
pub struct PlayerRedis {
    pub name: UserName,
    #[serde(default, skip_serializing_if = "negate_ref")]
    pub withdraw: bool,
    pub sheet: SheetScores,
    #[serde(default)]
    pub fire: bool,
    #[serde(flatten)]
    rest: JsValue,
}

impl PlayerRedis {
    fn expand(self, rank: usize) -> Player {
        Player {
            name: self.name,
            withdraw: self.withdraw,
            sheet: Sheet {
                fire: self.fire,
                scores: self.sheet,
            },
            rank,
            rest: self.rest,
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArenaFullRedis {
    pub id: ArenaId,
    #[serde(flatten)]
    pub shared: Arc<ArenaShared>,
    pub ongoing_user_games: HashMap<UserId, GameId>,
    pub standing: Vec<PlayerRedis>,
}

impl ArenaFullRedis {
    pub fn expand(self) -> ArenaFull {
        ArenaFull {
            id: self.id,
            ongoing_user_games: self.ongoing_user_games,
            ranking: standing_to_ranking(&self.standing),
            withdrawn: standing_to_withdrawn(&self.standing),
            standing: self
                .standing
                .into_iter()
                .enumerate()
                .map(|(index, p)| p.expand(index + 1))
                .collect(),
            shared: self.shared,
        }
    }
}

fn standing_to_ranking(standing: &[PlayerRedis]) -> FullRanking {
    FullRanking(
        standing
            .iter()
            .enumerate()
            .map(|(index, player)| (player.name.to_id(), Rank(index + 1)))
            .collect(),
    )
}

fn standing_to_withdrawn(standing: &[PlayerRedis]) -> HashSet<UserId> {
    standing
        .iter()
        .filter(|p| p.withdraw)
        .map(|p| p.name.to_id())
        .collect()
}
