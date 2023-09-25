use std::env::temp_dir;
use std::sync::{Arc, Mutex};

use criterion::BenchmarkId;
use criterion::{black_box, Criterion};
use criterion::{criterion_group, criterion_main};
use rand::prelude::*;
use tokio::runtime::Builder;
use uuid::Uuid;

use eemod::crypto;
use eemod::crypto::prelude::*;
use eemod::db::Db;
use eemod::eems::{EemsForSendImpl, PubInfo, PubSrc};
use eemod::grpc::eems::eems_for_send_client::EemsForSendClient;
use eemod::grpc::eems::GenIdReq;
use eemod::user::Sender;

fn gen_id_by_user(sender: &Sender, body_size: usize) -> GenIdReq {
    let msg_key: SymK = thread_rng().gen();
    let mut body = vec![0; body_size];
    thread_rng().fill_bytes(&mut body);
    let (req, _) = sender.gen_id_by_user(&body, &msg_key);
    req
}

fn from_body_size(c: &mut Criterion) {
    let rt = Builder::new_multi_thread().enable_all().build().unwrap();

    let sender_id_s = "f7584d00-6948-4e9c-b444-ff757f4dd9c1";
    let sender_id = Uuid::parse_str(sender_id_s).unwrap();
    let sender_sk: SK = thread_rng().gen();
    let sender_id_sign = crypto::pk_sign(&sender_sk, sender_id.as_bytes());
    let eems_addr = "127.0.0.1:8000";
    let eems_url = format!("http://{eems_addr}");
    let eems_client = rt.block_on(async { EemsForSendClient::connect(eems_url).await.unwrap() });
    let sender = Sender::new(sender_id, sender_id_sign, eems_client);

    let rocksdb_path = temp_dir().join("test-gen_id_by_entities");
    let rocksdb_inner = rocksdb::DB::open_default(rocksdb_path).unwrap();
    let db = Db::new(Arc::new(Mutex::new(rocksdb_inner)));
    let sk: SK = thread_rng().gen();
    let sender_pk: PK = crypto::pk_pk(&sender_sk);
    let pub_src = Box::new(PubSrcImpl::new(sender_pk));
    let eems = EemsForSendImpl::new(sk, pub_src, db);

    let body_size_iter = (3..13).into_iter().map(|x| 2usize.pow(x));
    body_size_iter.clone().for_each(|body_size| {
        c.bench_with_input(
            BenchmarkId::new("gen_id_by_user", body_size),
            &body_size,
            |b, &body_size| {
                b.iter(|| black_box(gen_id_by_user(&sender, body_size)));
            },
        );
    });

    {
        let body_size = 2usize.pow(12);
        let req = gen_id_by_user(&sender, body_size);

        c.bench_with_input(
            BenchmarkId::new("gen_id_by_eems", body_size),
            &body_size,
            |b, _| {
                b.iter(|| {
                    black_box(eems.gen_id_by_eems(&sender_id, &req.ct_hash, req.ct_hash_ct.clone()))
                });
            },
        );
    };
}

criterion_group!(benches, from_body_size);
criterion_main!(benches);

struct PubSrcImpl {
    pk: PK,
}

impl PubSrcImpl {
    pub fn new(pk: PK) -> Self {
        Self { pk }
    }
}

impl PubSrc for PubSrcImpl {
    fn get(&self, _id: &Uuid) -> Option<PubInfo> {
        Some(PubInfo { pk: self.pk })
    }
}
