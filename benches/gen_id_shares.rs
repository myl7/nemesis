use criterion::{black_box, Criterion};
use criterion::{criterion_group, criterion_main};
use rand::prelude::*;

use eemod::grpc::user::IdShare;
use eemod::user::Reporter;

const ID_SIZE: usize = 164;

fn gen_shares() -> [IdShare; 2] {
    let prg_key: [u8; 32] = thread_rng().gen();
    let reporter = Reporter::new(&prg_key);
    let mut id = vec![0; ID_SIZE];
    thread_rng().fill_bytes(&mut id);
    let s0s: [[u8; 16]; 2] = thread_rng().gen();
    let shares = reporter.gen_id_shares(&id, [&s0s[0], &s0s[1]]);
    shares
}

fn bench(c: &mut Criterion) {
    c.bench_function("gen_id_shares", |b| {
        b.iter(|| black_box(gen_shares()));
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
