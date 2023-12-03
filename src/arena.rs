use std::{
    collections::{HashMap, HashSet},
    ops::Not,
    str::FromStr,
};

use arrayvec::ArrayString;
use itertools::Itertools as _; // for flatten_ok
use serde::{Deserialize, Serialize};
use serde_json::Value as JsValue;
use serde_with::skip_serializing_none;
use thiserror::Error;

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
pub struct UserId(Box<str>);
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserName(Box<str>);
impl UserName {
    pub fn into_id(mut self) -> UserId {
        self.0.make_ascii_lowercase();
        UserId(self.0)
    }
}
#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct GameId(ArrayString<8>);
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct TeamId(Box<str>);
#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct Rank(pub usize);
#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct PauseSeconds(u32);

#[derive(Debug)]
pub struct ArenaFull {
    pub id: ArenaId,
    pub shared: ArenaShared,
    pub ongoing_user_games: OngoingUserGames,
    pub player_vec: Vec<Player>,
    pub player_map: HashMap<UserId, PlayerMapEntry>,
    pub withdrawn: HashSet<UserId>,
    pub team_standing: Option<TeamStanding>,
    pub pauses: HashMap<UserId, PauseSeconds>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[skip_serializing_none]
struct ClientMe {
    rank: Rank,
    #[serde(skip_serializing_if = "Not::not")]
    withdraw: bool,
    game_id: Option<GameId>,
    pause_delay: Option<PauseSeconds>,
}

#[skip_serializing_none]
#[derive(Serialize, Clone, Debug)]
pub struct Player {
    pub name: UserName,
    #[serde(skip_serializing_if = "Not::not")]
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
    #[serde(default, skip_serializing_if = "Not::not")]
    pub fire: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SheetScores(Box<str>);

#[derive(Debug, Clone)]
pub struct OngoingUserGames(HashMap<UserId, GameId>);

#[derive(Error, Debug)]
#[error("could not parse ongoing games")]
pub struct InvalidOngoingGames;

impl FromStr for OngoingUserGames {
    type Err = InvalidOngoingGames;

    fn from_str(encoded: &str) -> Result<OngoingUserGames, InvalidOngoingGames> {
        Ok(OngoingUserGames(
            encoded
                .split(',')
                .filter(|line| !line.is_empty())
                .map(|enc| {
                    let (players, game) = enc.split_once('/').ok_or(InvalidOngoingGames)?;
                    let (p1, p2) = players.split_once('&').ok_or(InvalidOngoingGames)?;
                    let game_id = GameId(ArrayString::from(game).map_err(|_| InvalidOngoingGames)?);
                    Ok([(UserId(p1.into()), game_id), (UserId(p2.into()), game_id)])
                })
                .flatten_ok()
                .collect::<Result<_, InvalidOngoingGames>>()?,
        ))
    }
}

#[derive(Debug, Clone, Serialize)]
struct ClientStanding<'a> {
    page: usize,
    players: &'a [Player],
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientData<'a> {
    #[serde(flatten)]
    shared: &'a ArenaShared,
    me: Option<ClientMe>,
    standing: ClientStanding<'a>,
    team_standing: Option<&'a [Team]>,
    my_team: Option<&'a Team>, // only for large battles, if not included in `team_standing`
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
                pause_delay: full.pauses.get(uid).copied(),
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
            shared: &full.shared,
            me,
            standing: ClientStanding { page, players },
            team_standing: full
                .team_standing
                .as_ref()
                .and_then(|teams| teams.0.chunks(10).next()),
            my_team: user_id.and_then(|uid| ClientData::get_my_team_if_not_included(full, uid)),
        }
    }

    fn get_my_team_if_not_included<'a>(full: &'a ArenaFull, user_id: &UserId) -> Option<&'a Team> {
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
    }
}
