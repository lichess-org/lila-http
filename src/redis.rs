use crate::arena::ArenaFull;
use crate::repo::Repo;
use futures::stream::StreamExt;
use log::error;
use std::error::Error;
use std::sync::Arc;

pub fn subscribe(opt: crate::opt::Opt, repo: Arc<Repo>) -> Result<(), Box<dyn Error>> {
    let _ = tokio::spawn(async move {
        let client = redis::Client::open(opt.redis_url).unwrap();
        let subscribe_con = client.get_tokio_connection().await.unwrap();
        let mut pubsub = subscribe_con.into_pubsub();
        pubsub.subscribe("http-out").await.unwrap();
        let mut stream = pubsub.on_message();
        while let Some(msg) = stream.next().await {
            let payload: String = msg.get_payload().unwrap();
            let _: () = match serde_json::from_str::<ArenaFull>(&payload) {
                Err(err) => error!("{:?}", err.to_string()),
                Ok(full) => repo.put(full).await,
            };
        }
    });
    Ok(())
}
