use crate::arena::{ArenaFull, ArenaId, ArenaShared, FullRanking, GameId, Player, Rank, UserId};
use crate::repo::Repo;
use futures::stream::StreamExt;
use log::error;
use redis::RedisError;
use serde::Deserialize;
use serde_json::Error as SerdeJsonError;
use serde_json::{Value as JsValue, Value::Object as JsObject};
use serde_with::serde_as;
use std::collections::HashMap;
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

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArenaFullRedis {
    pub id: ArenaId,
    #[serde(flatten)]
    pub shared: Arc<ArenaShared>,
    pub ongoing_user_games: HashMap<UserId, GameId>,
    pub standing: Vec<Player>,
}

impl ArenaFullRedis {
    pub fn expand(self) -> ArenaFull {
        ArenaFull {
            id: self.id,
            shared: self.shared,
            ongoing_user_games: self.ongoing_user_games,
            ranking: standing_to_ranking(&self.standing),
            standing: self.standing,
        }
    }
}

fn standing_to_ranking(standing: &[Player]) -> FullRanking {
    FullRanking(
        standing
            .iter()
            .enumerate()
            .map(|(index, player)| (player.name.to_id(), Rank(index + 1)))
            .collect(),
    )
}
