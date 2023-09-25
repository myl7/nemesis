use std::env::{temp_dir, var};
use std::sync::{Arc, Mutex};

use tonic::transport::Server;
use uuid::Uuid;

use eemod::crypto::prelude::*;
use eemod::db::Db;
use eemod::eems::{EemsForSendImpl, PubInfo, PubSrc};
use eemod::grpc::eems::eems_for_send_server::EemsForSendServer;

#[tokio::main]
async fn main() {
    let rocksdb_path = temp_dir().join(env!("CARGO_BIN_NAME"));
    let rocksdb_inner = rocksdb::DB::open_default(rocksdb_path).unwrap();
    let db = Db::new(Arc::new(Mutex::new(rocksdb_inner)));

    let sk: SK = hex::decode(var("EEMOD_EVAL_EEMS_SK").unwrap())
        .unwrap()
        .try_into()
        .unwrap();

    let user_pk_s = var("EEMOD_EVAL_SENDER_PK").unwrap();
    let user_pk: PK = hex::decode(user_pk_s).unwrap().try_into().unwrap();
    let pub_src = Box::new(PubSrcImpl::new(user_pk));

    let eems = EemsForSendImpl::new(sk, pub_src, db);

    let addr = "127.0.0.1:8000";
    println!("To listen on {addr}");
    Server::builder()
        .add_service(EemsForSendServer::new(eems))
        .serve(addr.parse().unwrap())
        .await
        .unwrap();
}

pub struct PubSrcImpl {
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
