use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    thread, time,
};

use futures::stream::StreamExt;
use log::error;
use serde::Deserialize;
use serde_json::{Error as SerdeJsonError, Value as JsValue};
use serde_with::{serde_as, FromInto};

use crate::{
    arena::{
        ArenaFull, ArenaId, ArenaShared, OngoingUserGames, PauseSeconds, Player, PlayerMapEntry,
        Rank, Sheet, SheetScores, TeamId, TeamStanding, UserId, UserName,
    },
    opt::RedisOpt,
    repo::Repo,
};

fn parse_message(msg: &redis::Msg) -> Result<ArenaFullRedis, SerdeJsonError> {
    serde_json::from_slice(msg.get_payload_bytes())
}

pub async fn subscribe(opt: RedisOpt, repo: &'static Repo) {
    let client = redis::Client::open(opt.redis_url.clone()).expect("valid redis url");
    loop {
        println!("Reddit stream connecting...");
        match client.get_tokio_connection().await {
            Ok(subscribe_con) => {
                let mut pubsub = subscribe_con.into_pubsub();
                pubsub.subscribe("http-out").await.unwrap();
                let mut stream = pubsub.on_message();
                println!("Reddit stream connected.");
                while let Some(msg) = stream.next().await {
                    match parse_message(&msg) {
                        Ok(full) => repo.put(full.expand()).await,
                        Err(msg) => error!("{:?}", msg),
                    }
                }
                println!("Reddit stream end!");
            }
            Err(error) => {
                println!("Couldn't connect to redis: {}", error);
            }
        }
        thread::sleep(time::Duration::from_secs(1));
    }
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
    pub pause: Option<PauseSeconds>,
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
    // TODO: Can probably remove Arc here, and just let ClientData borrow.
    // Can also disable rc feature for serde, when this is done.
    #[serde(flatten)]
    pub shared: Arc<ArenaShared>,
    #[serde_as(as = "FromInto<String>")]
    pub ongoing_user_games: OngoingUserGames,
    pub standing: Vec<PlayerRedis>,
    pub team_standing: Option<TeamStanding>,
}

impl ArenaFullRedis {
    pub fn expand(self) -> ArenaFull {
        let withdrawn = standing_to_withdrawn(&self.standing);
        let pauses = standing_to_pauses(&self.standing);
        let player_vec: Vec<Player> = self
            .standing
            .into_iter()
            .enumerate()
            .map(|(index, player)| player.expand(Rank(index + 1)))
            .collect();
        ArenaFull {
            id: self.id,
            ongoing_user_games: self.ongoing_user_games,
            withdrawn,
            player_map: make_player_map(&player_vec),
            player_vec,
            team_standing: self.team_standing,
            shared: self.shared,
            pauses,
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

fn standing_to_pauses(standing: &[PlayerRedis]) -> HashMap<UserId, PauseSeconds> {
    standing
        .into_iter()
        .flat_map(|player| {
            player
                .pause
                .map(|pause| (player.name.clone().into_id(), pause))
        })
        .collect()
}
