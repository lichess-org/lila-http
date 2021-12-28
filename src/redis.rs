// use redis::aio::Connection;

use futures::stream::StreamExt;
// use serde::Serialize;
use serde_json::Value as JsValue;
use std::error::Error;

// pub struct Redis {
//     publish_con: Connection,
// }

// impl Redis {
// pub async fn new(opt: crate::opt::Opt) -> Result<Self, RedisError> {
//     let client: Client = Client::open(opt.redis_url)?;
//     let publish_con: Connection = client.get_tokio_connection().await?;
//     Ok(Redis { publish_con })
// }

pub fn subscribe(opt: crate::opt::Opt) -> Result<(), Box<dyn Error>> {
    let _ = tokio::spawn(async move {
        let client = redis::Client::open(opt.redis_url).unwrap();
        let subscribe_con = client.get_tokio_connection().await.unwrap();
        let mut pubsub = subscribe_con.into_pubsub();
        pubsub.subscribe("http-out").await.unwrap();
        let mut stream = pubsub.on_message();
        while let Some(msg) = stream.next().await {
            // let msg = stream.next().await.unwrap();
            let payload: String = msg.get_payload().unwrap();
            let json = serde_json::from_str::<JsValue>(&payload).unwrap();
            dbg!(json);
        }
    });
    Ok(())
}

// we shouldn't need to tell lila anything (?)
//     pub async fn publish<M: Serialize>(&mut self, message: &M) -> Result<(), Box<dyn Error>> {
//         let json = serde_json::to_string(message).unwrap();
//         self.publish_con.publish("http-in", json).await?;
//         Ok(())
//     }
// }
