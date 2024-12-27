use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use dpf_dcf::prg::Aes256HirosePrg;
use dpf_dcf::{PointFn, Dpf as Dcf, DpfImpl as DcfImpl};
use dpf_dcf::group::byte::ByteGroup;
use rand::prelude::*;

fn from_item_num(c: &mut Criterion) {
    let prg_key: [u8; 32] = thread_rng().gen();
    let prg = Aes256HirosePrg::new([&prg_key]);
    let dcf = DcfImpl::new(prg);
    let f = PointFn::<16, 16, ByteGroup<16>> {
        alpha: 10u128.to_le_bytes(),
        beta: ByteGroup::from([0xff; 16]),
    };

    let item_num_iter = [6].into_iter().map(|x| 10usize.pow(x));
    item_num_iter.for_each(|item_num| {
        c.bench_with_input(
            BenchmarkId::new("gen_compare", item_num),
            &item_num,
            |b, _| {
                b.iter(|| {
for _ in 0..100 {
                    black_box({
                        let s00 = thread_rng().gen();
                        let s01 = thread_rng().gen();
                        let kappa_shares: Vec<_> = (0..item_num)
                            .map(|_| {
                                let mut share0 = dcf.gen(&f, [&s00, &s01]);
                                let mut share1 = share0.clone();
                                share0.s0s.remove(1);
                                share1.s0s.remove(0);
                                (share0, share1)
                            })
                            .collect();

                        let mut gammas = vec![0; item_num];
                        thread_rng().fill_bytes(&mut gammas);
                    })
                }
                });
            },
        );
    });
}

// criterion_group!(benches, from_item_num);
criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = from_item_num
}
criterion_main!(benches);
