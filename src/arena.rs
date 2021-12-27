use serde::Deserialize;

#[derive(Debug, Eq, PartialEq, Deserialize, Hash, Clone)]
pub struct ArenaId(pub String);

#[derive(Debug, Clone)]
pub struct ArenaFull {}
