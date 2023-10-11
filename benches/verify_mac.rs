use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use num_bigint::BigUint;
use rand::prelude::*;

use eemod::crypto::prelude::*;

fn from_item_num(c: &mut Criterion) {
    let item_num_iter = [1, 2, 5, 10, 12].into_iter().map(|x| x * 3600 * 11);
    let max_item_num = item_num_iter.clone().last().unwrap();

    // In the previous procedure we have evaluated the DCF, so now we just have a bunch of values to be summed up
    let values: Vec<u128> = (0..max_item_num).map(|_| thread_rng().gen()).collect();
    // `mk` is included in the ID already
    let mk: SymK = thread_rng().gen();
    // `gamma` and `tao` is provided by the reporter.
    // let gamma: SymK = thread_rng().gen();
    let tao: SymK = thread_rng().gen();

    item_num_iter.for_each(|item_num| {
        c.bench_with_input(
            BenchmarkId::new("verify_mac", item_num),
            &item_num,
            |b, _| {
                b.iter(|| {
                    black_box({
                        let mk_int = BigUint::from_bytes_le(&mk);
                        // let gamma_int = BigUint::from_bytes_le(&gamma);
                        let tao_int = BigUint::from_bytes_le(&tao);

                        let tp: SymK = thread_rng().gen();
                        let tp_int = BigUint::from_bytes_le(&tp);

                        let tp_sum_int = values[..item_num]
                            .iter()
                            .fold(tp_int, |lhs, rhs| lhs + BigUint::from(*rhs) * &mk_int);

                        // We should wrapping minus it.
                        // But it is hard for the crate `num_bigint` to do so (maybe).
                        // So we just do a minus to test the performance.
                        tp_sum_int - tao_int
                    })
                });
            },
        );
    });
}

criterion_group!(benches, from_item_num);
criterion_main!(benches);
