use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use futures::stream::StreamExt;
use log::error;
use serde::Deserialize;
use serde_json::{Error as SerdeJsonError, Value as JsValue};
use serde_with::{serde_as, FromInto};

use crate::{
    arena::{
        ArenaFull, ArenaId, ArenaShared, OngoingUserGames, Player, PlayerMapEntry, Rank, Sheet,
        SheetScores, TeamId, TeamStanding, UserId, UserName,
    },
    opt::RedisOpt,
    repo::Repo,
};

fn parse_message(msg: &redis::Msg) -> Result<ArenaFullRedis, SerdeJsonError> {
    serde_json::from_slice(msg.get_payload_bytes())
}

pub async fn subscribe(opt: RedisOpt, repo: &'static Repo) {
    let client = redis::Client::open(opt.redis_url).unwrap();
    let subscribe_con = client.get_tokio_connection().await.unwrap();
    let mut pubsub = subscribe_con.into_pubsub();
    pubsub.subscribe("http-out").await.unwrap();
    let mut stream = pubsub.on_message();
    while let Some(msg) = stream.next().await {
        match parse_message(&msg) {
            Ok(full) => repo.put(full.expand()).await,
            Err(msg) => error!("{:?}", msg),
        }
    }
    // TODO: What's the best we can do when the Redis connection dies?
}

#[derive(Deserialize, Clone, Debug)]
struct PlayerRedis {
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
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ArenaFullRedis {
    pub id: ArenaId,
    #[serde(flatten)]
    pub shared: Arc<ArenaShared>,
    #[serde_as(as = "FromInto<String>")]
    pub ongoing_user_games: OngoingUserGames,
    pub standing: Vec<PlayerRedis>,
    #[serde(default)]
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

fn make_player_map(standing: &[Player]) -> HashMap<UserId, PlayerMapEntry> {
    standing
        .iter()
        .map(|player| {
            (
                player.name.clone().into_id(),
                PlayerMapEntry {
                    rank: player.rank,
                    team: player.team.clone(),
                },
            )
        })
        .collect()
}

fn standing_to_withdrawn(standing: &[PlayerRedis]) -> HashSet<UserId> {
    standing
        .iter()
        .filter(|p| p.withdraw)
        .map(|p| p.name.clone().into_id())
        .collect()
}
