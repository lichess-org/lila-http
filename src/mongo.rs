use crate::arena::{ArenaId, UserId};
use mongodb::{bson::doc, options::ClientOptions, Client, Collection, Database};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    withdraw: bool,
    team: Option<String>,
}

pub struct Mongo {
    client: Client,
    db: Database,
    players: Collection<Player>,
}

impl Mongo {
    pub async fn new(opt: crate::opt::Opt) -> Result<Mongo, mongodb::error::Error> {
        let client = Client::with_options(ClientOptions::parse(opt.mongo_url).await?)?;
        let db = client.database(&opt.mongo_db_name);
        Ok(Mongo {
            client: client.clone(),
            db: db.clone(),
            players: db.collection::<Player>("tournament_player"),
        })
    }

    pub async fn get_player(&self, tour_id: ArenaId, user_id: UserId) -> Option<Player> {
        let filter = doc! { "tid": tour_id.0, "uid": user_id.0 };
        // let find_options = FindOptions::builder().build();
        self.players.find_one(filter, None).await.unwrap_or(None)
    }
}
