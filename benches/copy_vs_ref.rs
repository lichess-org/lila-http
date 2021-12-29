use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lila_http::arena::{
    ArenaFull, ArenaId, ArenaShared, ClientData, ClientMe, FullRanking, UserId,
};
use serde::Serialize;
use serde_json::{json, to_string, Value as JsValue};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize)]
struct ClientStandingRef<'p> {
    page: u32,
    players: &'p [JsValue],
}

#[derive(Debug, Clone, Serialize)]
pub struct ClientDataRef<'a> {
    #[serde(flatten)]
    shared: &'a ArenaShared,
    #[serde(skip_serializing_if = "Option::is_none")]
    me: Option<ClientMe>,
    standing: ClientStandingRef<'a>,
}

impl<'a> ClientDataRef<'a> {
    pub fn new<'b>(
        page: usize,
        full: &'b Arc<ArenaFull>,
        user_id: Option<UserId>,
    ) -> ClientDataRef<'b> {
        ClientDataRef {
            shared: &full.shared,
            me: user_id.map(|uid| ClientMe {
                rank: full.ranking.ranking.get(&uid).cloned(),
                withdraw: false, // todo!(),
                game_id: full.ongoing_user_games.get(&uid).cloned(),
                pause_delay: None,
            }),
            standing: ClientStandingRef {
                page: 1,
                players: &full.standing[((page - 1) * 10)..(page * 10 - 1)], // TODO: check bounds,
            },
        }
    }
}

fn no_ref(full: Arc<ArenaFull>, user_id: Option<UserId>, page: usize) -> String {
    to_string(&ClientData::new(page, full, user_id)).unwrap()
}

fn with_ref(full: &Arc<ArenaFull>, user_id: Option<UserId>, page: usize) -> String {
    to_string(&ClientDataRef::new(page, full, user_id)).unwrap()
}

fn generate_arena(id: ArenaId) -> ArenaFull {
    let shared = Arc::new(ArenaShared {
        nb_players: 15000,
        duels: JsValue::Null,
        seconds_to_finish: Some(200000),
        seconds_to_start: Some(200000),
        is_started: Some(true),
        is_finished: Some(false),
        is_recently_finished: Some(false),
        featured: Some(JsValue::Null),
        podium: Some(JsValue::Null),
        pairings_closed: Some(false),
        stats: Some(JsValue::Null),
        team_standing: Some(JsValue::Null),
        duel_teams: Some(JsValue::Null),
    });
    let ongoing_user_games = HashMap::new();
    let ranking = FullRanking {
        ranking: HashMap::new(),
    };
    let mut standing = Vec::new();
    let rando: Vec<JsValue> = (1..1000i64).map(|i| json!(i)).collect();
    for _i in 1..1500 {
        standing.push(JsValue::Array(rando.clone()));
    }
    ArenaFull {
        id,
        shared,
        ongoing_user_games,
        ranking,
        standing,
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let user_id = Some(UserId("asdf".into()));

    let file = File::open("/home/lakin/Downloads/ArenaFull-marathon.json").unwrap();
    let reader = BufReader::new(file);
    let full: ArenaFull = serde_json::from_reader(reader).unwrap();

    let full = Arc::new(full);
    for i in 1..10 {
        c.bench_function(format!("copy {}", i).as_str(), |b| {
            b.iter(|| {
                no_ref(
                    black_box(full.clone()),
                    black_box(user_id.clone()),
                    black_box(i),
                )
            })
        });
        c.bench_function(format!("with ref {}", i).as_str(), |b| {
            b.iter(|| with_ref(black_box(&full), black_box(user_id.clone()), black_box(i)))
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
