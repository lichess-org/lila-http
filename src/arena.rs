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
    nb_players: u32,
    duels: JsValue,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    seconds_to_finish: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    seconds_to_start: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    is_started: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    is_finished: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    is_recently_finished: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    featured: Option<JsValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    podium: Option<JsValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pairings_closed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    stats: Option<JsValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    team_standing: Option<JsValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    duel_teams: Option<JsValue>,
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
    shared: Arc<ArenaShared>,
    ongoing_user_games: HashMap<UserId, GameId>,
    // this duplicates info gotten from standing, remove
    #[serde_as(as = "FromInto<String>")]
    ranking: FullRanking,
    standing: Vec<JsValue>,
}

#[derive(Debug, Clone)]
struct FullRanking {
    ranking: HashMap<UserId, Rank>,
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
struct ClientMe {
    rank: Option<Rank>,
    withdraw: bool,
    game_id: Option<GameId>,
    pause_delay: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
struct ClientStanding {
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
    pub fn new(full: Arc<ArenaFull>, user_id: Option<UserId>) -> ClientData {
        let page = 1;
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
