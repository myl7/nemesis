use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use dcf::{Cw, Share};
use group_math::int::U128Group;
use rand::prelude::*;

use eemod::grpc::user::{IdShare, IdShareCw};
use eemod::msp::MspModeration;
use eemod::user::Reporter;

fn from_item_num(c: &mut Criterion) {
    let item_num_iter = [1, 2, 5, 10, 12].into_iter().map(|x| x * 3600 * 11);
    let max_item_num = item_num_iter.clone().last().unwrap();

    let prg_key: [u8; 32] = thread_rng().gen();
    let reporter = Reporter::new(&prg_key);
    const ID_SIZE: usize = 164;
    let mut id = vec![0; ID_SIZE];
    let party = true;
    let shares = [{
        thread_rng().fill_bytes(&mut id);
        let s0s: [[u8; 16]; 2] = thread_rng().gen();
        let [share0, share1] = reporter.gen_id_shares(&id, [&s0s[0], &s0s[1]]);
        let IdShare { s0, cw_np1, cws } = if party { share0 } else { share1 };
        Share {
            s0s: vec![s0.try_into().unwrap()],
            cw_np1: TryInto::<[u8; 16]>::try_into(cw_np1).unwrap().into(),
            cws: cws
                .into_iter()
                .map(|cw| {
                    let IdShareCw { s, tl, tr } = cw;
                    Cw {
                        s: s.try_into().unwrap(),
                        // TODO: Fix type inference for LAMBDA
                        v: U128Group(0),
                        tl,
                        tr,
                    }
                })
                .collect(),
        }
    }];

    let dpf_prg_key: [u8; 32] = thread_rng().gen();
    let dcf_prg_key: [u8; 32] = thread_rng().gen();
    let mut id_hashes = vec![[0u8; 32]; max_item_num];
    id_hashes.iter_mut().for_each(|x| {
        rand::thread_rng().fill_bytes(x);
    });

    item_num_iter.for_each(|item_num| {
        let msp_mod = MspModeration::new(
            party,
            &dcf_prg_key,
            &dpf_prg_key,
            id_hashes[..item_num].to_vec(),
        );

        c.bench_with_input(
            BenchmarkId::new("update_msp", item_num),
            &item_num,
            |b, _| {
                b.iter(|| {
                    black_box({
                        let mut values = vec![0; item_num];
                        msp_mod.sum_report(&shares, &mut values);
                        values
                    })
                });
            },
        );
    });
}

criterion_group!(benches, from_item_num);
criterion_main!(benches);
