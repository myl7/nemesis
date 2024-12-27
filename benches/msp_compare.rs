use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
// use dcf::prg::Aes256HirosePrg;
// use dcf::{BoundState, CmpFn, Dcf, DcfImpl};
use dpf_dcf::prg::Aes256HirosePrg as DcfAes256HirosePrg;
use dpf_dcf::{Dpf as Dcf, DpfImpl as DcfImpl, Share, PointFn};
use dpf_dcf::group::byte::ByteGroup;

use group_math::int::U128Group;
use rand::prelude::*;

use eemod::msp::MspModeration;

fn from_item_num(c: &mut Criterion) {
    println!("actually 10**8");
    let item_num_iter = [7].into_iter().map(|x| 10usize.pow(x));
    let max_item_num = item_num_iter.clone().last().unwrap();

    let party = true;
    let prg_key: [u8; 32] = thread_rng().gen();
    let prg = DcfAes256HirosePrg::new([&prg_key]);
    let dcf = DcfImpl::new(prg);
    let f = PointFn::<16, 16, ByteGroup<16>> {
        alpha: 10u128.to_le_bytes(),
        beta: ByteGroup::from([0xff; 16]),
    };
    let s00 = thread_rng().gen();
    let s01 = thread_rng().gen();
    let kappa_shares: Vec<_> = (0..max_item_num)
        .map(|_| {
            let mut share = dcf.gen(&f, [&s00, &s01]);
            share.s0s.remove(if party { 0 } else { 1 });
            share
        })
        .collect();

    let mut gammas = vec![0; max_item_num];
    thread_rng().fill_bytes(&mut gammas);

    let dpf_prg_key: [u8; 32] = thread_rng().gen();
    let dcf_prg_key: [u8; 32] = thread_rng().gen();
    let mut id_hashes = vec![[0u8; 16]; max_item_num];
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
            BenchmarkId::new("msp_compare", item_num),
            &item_num,
            |b, _| {
                b.iter(|| {
                    for i in 0..10 {
                    black_box({
                        let mut values = vec![0; item_num];
                        msp_mod.check_threhold(
                            &kappa_shares[..item_num],
                            gammas
                                .iter()
                                .map(|&i| i as u128)
                                .take(item_num)
                                .collect::<Vec<_>>()
                                .as_ref(),
                            &mut values[..item_num],
                        );
                    })
                }
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
