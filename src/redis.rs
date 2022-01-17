use crate::arena::ArenaFull;
use crate::repo::Repo;
use futures::stream::StreamExt;
use log::error;
use redis::RedisError;
use serde_json::Error as SerdeJsonError;
use std::sync::Arc;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("[REDIS] Error getting payload: {0}")]
    RedisError(#[from] RedisError),
    #[error("[SERDE] Error parsing JSON: {0}")]
    SerdeJsonError(#[from] SerdeJsonError),
}

pub fn parse_message(msg: &redis::Msg) -> Result<ArenaFull, Error> {
    let str = &msg.get_payload::<String>()?;
    let res = serde_json::from_str(str)?;
    Ok(res)
}

pub fn subscribe(opt: crate::opt::Opt, repo: Arc<Repo>) -> Result<(), Error> {
    let _ = tokio::spawn(async move {
        let client = redis::Client::open(opt.redis_url).unwrap();
        let subscribe_con = client.get_tokio_connection().await.unwrap();
        let mut pubsub = subscribe_con.into_pubsub();
        pubsub.subscribe("http-out").await.unwrap();
        let mut stream = pubsub.on_message();
        while let Some(msg) = stream.next().await {
            match parse_message(&msg) {
                Ok(full) => repo.put(full).await,
                Err(msg) => error!("{:?}", dbg!(msg)),
            }
        }
    });
    Ok(())
}
