use criterion::BenchmarkId;
use criterion::{black_box, Criterion};
use criterion::{criterion_group, criterion_main};
use rand::prelude::*;

use eemod::crypto::prelude::*;
use eemod::msp::{EemsVec, MspIdShuffle, Perm, SymEncPerm};
use eemod::utils;

fn gen_vec(item_num: usize) -> Vec<Vec<u8>> {
    const ID_SIZE: usize = 164;
    let mut share = vec![vec![0u8; ID_SIZE]; item_num];
    share.iter_mut().for_each(|x| {
        rand::thread_rng().fill_bytes(x);
    });
    share
}

fn from_item_num(c: &mut Criterion) {
    let item_num_iter = [4, 5, 6, 7, 8].into_iter().map(|x| 10usize.pow(x));
    item_num_iter.for_each(|item_num| {
        // TODO: Replace with the impl in `eems`
        let share1 = gen_vec(item_num);
        let share2 = gen_vec(item_num);
        let a1 = gen_vec(item_num);
        let a2 = gen_vec(item_num);
        let a3 = gen_vec(item_num);
        let shared_perm_key: SymK = rand::thread_rng().gen();
        let shared_perm = SymEncPerm::new(shared_perm_key);
        let eems_perm_key1: SymK = rand::thread_rng().gen();
        let eems_perm1 = SymEncPerm::new(eems_perm_key1);
        let eems_perm_key2: SymK = rand::thread_rng().gen();
        let eems_perm2 = SymEncPerm::new(eems_perm_key2);

        let mut delta = a3.clone();
        delta.iter_mut().enumerate().for_each(|(i, item)| {
            eems_perm1.perm(item);
            utils::bytes_add(item, &a1[i]);
            eems_perm2.perm(item);
            utils::bytes_minus(item, &a2[i]);
        });

        let mut shuffle1 =
            MspIdShuffle::new(share1, shared_perm.clone(), eems_perm1, EemsVec::OneVec(a1));
        let mut shuffle2 = MspIdShuffle::new(
            share2,
            shared_perm,
            eems_perm2,
            EemsVec::ThreeVec(a2, a3, delta),
        );

        c.bench_with_input(
            BenchmarkId::new("shuffle_id_gen_z2_by_party2", item_num),
            &item_num,
            |b, _| {
                b.iter(|| {
                    black_box({
                        let mut shuffle = shuffle2.clone();
                        shuffle.run_msp_perm();
                        shuffle.gen_shared_share(None)
                    })
                });
            },
        );

        shuffle2.run_msp_perm();
        let z2 = shuffle2.gen_shared_share(None);

        c.bench_with_input(
            BenchmarkId::new("shuffle_id_gen_z1_by_party1", item_num),
            &item_num,
            |b, _| {
                b.iter(|| {
                    black_box({
                        let mut shuffle = shuffle1.clone();
                        shuffle.run_msp_perm();
                        shuffle.gen_shared_share(Some(&z2))
                    })
                });
            },
        );

        shuffle1.run_msp_perm();
        let z1 = shuffle1.gen_shared_share(Some(&z2));

        c.bench_with_input(
            BenchmarkId::new("shuffle_id_gen_id_by_party2", item_num),
            &item_num,
            |b, _| {
                b.iter(|| {
                    black_box({
                        let shuffle = shuffle2.clone();
                        shuffle.gen_shared_share(Some(&z1))
                    })
                });
            },
        );
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = from_item_num
}
criterion_main!(benches);
