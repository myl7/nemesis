use criterion::BenchmarkId;
use criterion::{black_box, Criterion};
use criterion::{criterion_group, criterion_main};
use rand::prelude::*;

use eemod::crypto::prelude::*;
use eemod::eems::EemsIdShuffle;
use eemod::msp::SymEncPerm;

fn gen_vec(item_num: usize) -> Vec<Vec<u8>> {
    const ID_SIZE: usize = 164;
    let mut share = vec![vec![0u8; ID_SIZE]; item_num];
    share.iter_mut().for_each(|x| {
        rand::thread_rng().fill_bytes(x);
    });
    share
}

fn from_item_num(c: &mut Criterion) {
    let item_num_iter = (4..7).into_iter().map(|x| 10usize.pow(x));
    item_num_iter.for_each(|item_num| {
        let shared_perm_key: SymK = rand::thread_rng().gen();
        let shared_perm = SymEncPerm::new(shared_perm_key);
        let ids = gen_vec(item_num);

        c.bench_with_input(
            BenchmarkId::new("gen_id_shuffle", item_num),
            &item_num,
            |b, _| {
                b.iter(|| {
                    black_box({
                        let shuffled_ids = ids.clone();
                        let shuffle_gen = EemsIdShuffle::new(shared_perm.clone());
                        shuffle_gen.gen_shuffle_shares(item_num, shuffled_ids)
                    })
                });
            },
        );
    });
}

criterion_group!(benches, from_item_num);
criterion_main!(benches);
