use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::prelude::*;

use eemod::crypto::prelude::*;
use eemod::user::ReporterGenMac;

fn bench(c: &mut Criterion) {
    let mk: SymK = thread_rng().gen();
    let report_gen = ReporterGenMac::new();

    c.bench_function("gen_mac", |b| {
        b.iter(|| black_box(report_gen.gen(&mk)));
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
