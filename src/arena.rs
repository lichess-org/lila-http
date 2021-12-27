use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Deserialize, Hash, Clone)]
pub struct ArenaId(pub String);

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArenaFull {
    nb_players: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClientData {
    nb_players: u32,
}

impl ClientData {
    pub fn new(full: ArenaFull) -> ClientData {
        ClientData {
            nb_players: full.nb_players,
        }
    }
}
