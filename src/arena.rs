use serde::{Deserialize, Serialize};
use serde_json::Value as JsValue;
use serde_with::{serde_as, FromInto};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Eq, PartialEq, Deserialize, Hash, Clone)]
pub struct ArenaId(pub String);

// naming is hard
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArenaShared {
    pub nb_players: u32,
    pub duels: JsValue,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds_to_finish: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds_to_start: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_started: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_finished: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_recently_finished: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub featured: Option<JsValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub podium: Option<JsValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pairings_closed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stats: Option<JsValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_standing: Option<JsValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duel_teams: Option<JsValue>,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Hash)]
pub struct UserId(pub String);
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameId(String);
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rank(usize);

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArenaFull {
    pub id: ArenaId,
    #[serde(flatten)]
    pub shared: Arc<ArenaShared>,
    pub ongoing_user_games: HashMap<UserId, GameId>,
    // this duplicates info gotten from standing, remove
    #[serde_as(as = "FromInto<String>")]
    pub ranking: FullRanking,
    pub standing: Vec<JsValue>,
}

#[derive(Debug, Clone)]
pub struct FullRanking {
    pub ranking: HashMap<UserId, Rank>,
}

impl From<String> for FullRanking {
    fn from(user_ids_comma_separated: String) -> Self {
        let user_ids = user_ids_comma_separated
            .split(',')
            .into_iter()
            .map(|uid| UserId(uid.to_string()));
        FullRanking {
            ranking: user_ids
                .enumerate()
                .map(|(index, uid)| (uid, Rank(index + 1)))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ClientMe {
    pub rank: Option<Rank>,
    pub withdraw: bool,
    pub game_id: Option<GameId>,
    pub pause_delay: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClientStanding {
    page: u32,
    players: Vec<JsValue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClientData {
    #[serde(flatten)]
    shared: Arc<ArenaShared>,
    #[serde(skip_serializing_if = "Option::is_none")]
    me: Option<ClientMe>,
    standing: ClientStanding,
}

impl ClientData {
    pub fn new(page: usize, full: Arc<ArenaFull>, user_id: Option<UserId>) -> ClientData {
        ClientData {
            shared: Arc::clone(&full.shared),
            me: user_id.map(|uid| ClientMe {
                rank: full.ranking.ranking.get(&uid).cloned(),
                withdraw: false, // todo!(),
                game_id: full.ongoing_user_games.get(&uid).cloned(),
                pause_delay: None,
            }),
            standing: ClientStanding {
                page: 1,
                players: full.standing[((page - 1) * 10)..(page * 10 - 1)].to_vec(), // TODO: check bounds,
            },
        }
    }
}
