use std::env::temp_dir;
use std::sync::{Arc, Mutex};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use prost::Message;
use rand::prelude::*;
use uuid::Uuid;

use eemod::crypto;
use eemod::crypto::prelude::*;
use eemod::db::{Db, IdArchive};
use eemod::eems::EemsSource;
use eemod::grpc::eems::GenIdSignedPack;
use eemod::grpc::user::MsgId;

fn from_body_size(c: &mut Criterion) {
    let msg_key: SymK = thread_rng().gen();
    let id_key: SymK = thread_rng().gen();
    let src = Uuid::from_bytes(thread_rng().gen());
    let src_ct = crypto::sym_enc(&id_key, src.as_bytes());
    let eems_sk: SK = thread_rng().gen();
    let eems_pk: PK = crypto::pk_pk(&eems_sk);
    let rocksdb_path = temp_dir().join("test-seek_source");

    let body_size_iter = (3..13).into_iter().map(|x| 2usize.pow(x));
    body_size_iter.for_each(|body_size| {
        c.bench_with_input(
            BenchmarkId::new("seek_source", body_size),
            &body_size,
            |b, &body_size| {
                let mut body = vec![0; body_size];
                thread_rng().fill_bytes(&mut body);
                let ct = crypto::sym_enc(&msg_key, &body);
                let ct_hash = crypto::hash(&ct);
                let ct_hash_ct = crypto::sym_enc(&msg_key, &ct_hash);

                let signed_buf = GenIdSignedPack {
                    src_ct: src_ct.clone(),
                    ct_hash: ct_hash.to_vec(),
                    ct_hash_ct: ct_hash_ct.clone(),
                }
                .encode_to_vec();
                let sign = crypto::pk_sign(&eems_sk, &signed_buf);

                let rocksdb_inner = rocksdb::DB::open_default(&rocksdb_path).unwrap();
                let db = Db::new(Arc::new(Mutex::new(rocksdb_inner)));
                db.put_id_archive_sync(
                    &ct_hash,
                    IdArchive {
                        id_key: id_key.clone(),
                        ct: ct.clone(),
                    },
                );

                let msg_id = MsgId {
                    msg_key: msg_key.to_vec(),
                    ct_hash: ct_hash.to_vec(),
                    src_ct: src_ct.clone(),
                    sign: sign.to_vec(),
                };
                let eems_source = EemsSource::new(eems_pk, db);

                b.iter(|| {
                    black_box({
                        eems_source.seek_source(&msg_id).unwrap();
                    })
                });
            },
        );
    });
}

criterion_group!(benches, from_body_size);
criterion_main!(benches);
