use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use arrayvec::ArrayString;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsValue;
use serde_with::skip_serializing_none;

#[derive(Debug, Eq, PartialEq, Deserialize, Hash, Copy, Clone)]
pub struct ArenaId(pub ArrayString<8>);

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArenaShared {
    nb_players: u32,
    duels: JsValue,
    seconds_to_finish: Option<u32>,
    seconds_to_start: Option<u32>,
    is_started: Option<bool>,
    is_finished: Option<bool>,
    is_recently_finished: Option<bool>,
    featured: Option<JsValue>,
    podium: Option<JsValue>,
    pairings_closed: Option<bool>,
    stats: Option<JsValue>,
    duel_teams: Option<JsValue>,
}

#[derive(Debug, Deserialize)]
pub struct TeamStanding(pub Vec<Team>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    id: TeamId,
    rank: Rank,
    #[serde(flatten)]
    rest: JsValue,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Hash)]
pub struct UserId(pub String);
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserName(pub String);
impl UserName {
    pub fn into_id(mut self) -> UserId {
        self.0.make_ascii_lowercase();
        UserId(self.0)
    }
}
#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct GameId(ArrayString<8>);
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct TeamId(String);
#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct Rank(pub usize);

#[derive(Debug)]
pub struct ArenaFull {
    pub id: ArenaId,
    pub shared: Arc<ArenaShared>,
    pub ongoing_user_games: OngoingUserGames,
    pub player_vec: Vec<Player>,
    pub player_map: HashMap<UserId, PlayerMapEntry>,
    pub withdrawn: HashSet<UserId>,
    pub team_standing: Option<TeamStanding>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ClientMe {
    rank: Rank,
    withdraw: bool,
    game_id: Option<GameId>,
    pause_delay: Option<u32>,
}

fn is_false(b: &bool) -> bool {
    !b
}

#[skip_serializing_none]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Player {
    pub name: UserName,
    #[serde(default, skip_serializing_if = "is_false")]
    pub withdraw: bool,
    pub sheet: Sheet,
    pub rank: Rank,
    pub team: Option<TeamId>,
    #[serde(flatten)]
    pub rest: JsValue,
}

#[derive(Debug)]
pub struct PlayerMapEntry {
    pub rank: Rank,
    pub team: Option<TeamId>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Sheet {
    pub scores: SheetScores,
    #[serde(default, skip_serializing_if = "is_false")]
    pub fire: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SheetScores(String);

#[derive(Debug, Clone)]
pub struct OngoingUserGames(HashMap<UserId, GameId>);

impl From<String> for OngoingUserGames {
    fn from(encoded: String) -> Self {
        OngoingUserGames(
            encoded
                .split(',')
                .filter(|line| !line.is_empty())
                .flat_map(|enc| {
                    let (players, game) = enc.split_once('/').unwrap();
                    let (p1, p2) = players.split_once('&').unwrap();
                    let game_id = GameId(ArrayString::from(game).unwrap());
                    [
                        (UserId(p1.to_string()), game_id),
                        (UserId(p2.to_string()), game_id),
                    ]
                })
                .collect(),
        )
    }
}

#[derive(Debug, Clone, Serialize)]
struct ClientStanding {
    page: usize,
    players: Vec<Player>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientData<'a> {
    #[serde(flatten)]
    shared: Arc<ArenaShared>,
    me: Option<ClientMe>,
    standing: ClientStanding,
    team_standing: Option<Vec<Team>>,
    my_team: Option<Team>, // only for large battles, if not included in `team_standing`
    #[serde(skip)]
    _todo_remove_this: &'a (),
}

impl ClientData<'_> {
    pub fn new<'a>(
        full: &'a ArenaFull,
        req_page: Option<usize>,
        user_id: Option<&UserId>,
    ) -> ClientData<'a> {
        let me = user_id.and_then(|uid| {
            full.player_map.get(uid).map(|player| ClientMe {
                rank: player.rank,
                withdraw: full.withdrawn.contains(uid),
                game_id: full.ongoing_user_games.0.get(uid).copied(),
                pause_delay: None, // TODO?
            })
        });
        let page = req_page
            .or_else(|| me.as_ref().map(|player| (player.rank.0 + 9) / 10))
            .unwrap_or(1);
        let players = full
            .player_vec
            .chunks(10)
            .nth(page.saturating_sub(1))
            .unwrap_or_default();

        ClientData {
            shared: Arc::clone(&full.shared),
            me,
            standing: ClientStanding {
                page,
                players: players.to_vec(),
            },
            team_standing: full
                .team_standing
                .as_ref()
                .map(|teams| teams.0.iter().take(10).cloned().collect()),
            my_team: user_id.and_then(|uid| ClientData::get_my_team_if_not_included(full, uid)),
            _todo_remove_this: &(),
        }
    }

    fn get_my_team_if_not_included(full: &ArenaFull, user_id: &UserId) -> Option<Team> {
        let player = full.player_map.get(user_id)?;
        let team_id = player.team.as_ref()?;
        let big_standing = full
            .team_standing
            .as_ref()
            .filter(|teams| teams.0.len() > 10)?;
        big_standing
            .0
            .iter()
            .find(|team| &team.id == team_id)
            .filter(|team| team.rank.0 > 10)
            .cloned()
    }
}
