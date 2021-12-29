use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lila_http::{
    arena::{ArenaFull, ArenaId, ArenaShared, ClientData, ClientMe, FullRanking, UserId},
    repo::Repo,
};
use serde::Serialize;
use serde_json::{json, Value as JsValue};
use std::collections::HashMap;
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
    pub fn new<'b>(full: &'b Arc<ArenaFull>, user_id: Option<UserId>) -> ClientDataRef<'b> {
        let page = 50;
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

fn no_ref(full: Arc<ArenaFull>, user_id: Option<UserId>) -> ClientData {
    ClientData::new(full, user_id)
}

fn with_ref<'a>(full: &'a Arc<ArenaFull>, user_id: Option<UserId>) -> ClientDataRef<'a> {
    ClientDataRef::new(full, user_id)
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
    let id = ArenaId("asdf".into());
    let user_id = Some(UserId("asdf".into()));

    let full = Arc::new(generate_arena(id));
    c.bench_function("no ref", |b| {
        b.iter(|| no_ref(black_box(full.clone()), black_box(user_id.clone())))
    });
    c.bench_function("with ref", |b| {
        b.iter(|| with_ref(black_box(&full), black_box(user_id.clone())))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
