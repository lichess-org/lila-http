use serde::Deserialize;

#[derive(Debug, Eq, PartialEq, Deserialize, Hash, Clone)]
pub struct ArenaId(pub String);

#[derive(Debug, Clone, Deserialize)]
pub struct ArenaFull {
    nb_players: u32,
}
