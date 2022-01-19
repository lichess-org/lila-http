use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use futures::stream::StreamExt;
use log::error;
use redis::RedisError;
use serde::Deserialize;
use serde_json::{Error as SerdeJsonError, Value as JsValue};
use serde_with::{serde_as, FromInto};
use thiserror::Error as ThisError;

use crate::{
    arena::{
        ArenaFull, ArenaId, ArenaShared, OngoingUserGames, Player, Rank, Sheet, SheetScores,
        TeamId, TeamStanding, UserId, UserName,
    },
    repo::Repo,
};

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
    #[serde(default)]
    pub withdraw: bool,
    pub sheet: SheetScores,
    #[serde(default)]
    pub fire: bool,
    pub team: Option<TeamId>,
    #[serde(flatten)]
    rest: JsValue,
}

impl PlayerRedis {
    fn expand(self, rank: Rank) -> Player {
        Player {
            name: self.name,
            withdraw: self.withdraw,
            sheet: Sheet {
                fire: self.fire,
                scores: self.sheet,
            },
            rank,
            team: self.team,
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
    #[serde_as(as = "FromInto<String>")]
    pub ongoing_user_games: OngoingUserGames,
    pub standing: Vec<PlayerRedis>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_standing: Option<TeamStanding>,
}

impl ArenaFullRedis {
    pub fn expand(self) -> ArenaFull {
        let withdrawn = standing_to_withdrawn(&self.standing);
        let player_vec: Vec<Player> = self
            .standing
            .into_iter()
            .enumerate()
            .map(|(index, p)| p.expand(Rank(index + 1)))
            .collect();
        ArenaFull {
            id: self.id,
            ongoing_user_games: self.ongoing_user_games,
            withdrawn,
            player_map: make_player_map(&player_vec),
            player_vec,
            team_standing: self.team_standing,
            shared: self.shared,
        }
    }
}

fn make_player_map(standing: &[Player]) -> HashMap<UserId, Player> {
    standing
        .iter()
        .map(|player| (player.name.to_id(), player.clone()))
        .collect()
}

fn standing_to_withdrawn(standing: &[PlayerRedis]) -> HashSet<UserId> {
    standing
        .iter()
        .filter(|p| p.withdraw)
        .map(|p| p.name.to_id())
        .collect()
}
