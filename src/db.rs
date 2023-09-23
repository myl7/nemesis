use std::sync::{Arc, Mutex};

use tokio::task::spawn_blocking;

use crate::crypto::prelude::*;

pub struct Db {
    inner: Arc<Mutex<rocksdb::DB>>,
}

impl Db {
    pub fn new(inner: Arc<Mutex<rocksdb::DB>>) -> Self {
        Self { inner }
    }

    pub async fn get_id_archive(&self, ct_hash: &Digest) -> Option<IdArchive> {
        let ct_hash = ct_hash.clone();
        let db_arc = self.inner.clone();
        spawn_blocking(move || {
            let db = db_arc.lock().unwrap();
            let id_key = db
                .get([b"id_archive/id_key/".as_ref(), &ct_hash].concat())
                .unwrap()?;
            let ct = db
                .get([b"id_archive/ct/".as_ref(), &ct_hash].concat())
                .unwrap()?;
            Some(IdArchive {
                id_key: id_key.try_into().unwrap(),
                ct,
            })
        })
        .await
        .unwrap()
    }

    pub async fn put_id_archive(&self, ct_hash: &Digest, id_ar: IdArchive) {
        let ct_hash = ct_hash.clone();
        let db_arc = self.inner.clone();
        spawn_blocking(move || {
            let db = db_arc.lock().unwrap();
            db.put(
                [b"id_archive/id_key/".as_ref(), &ct_hash].concat(),
                id_ar.id_key,
            )
            .unwrap();
            db.put([b"id_archive/ct/".as_ref(), &ct_hash].concat(), id_ar.ct)
                .unwrap();
        })
        .await
        .unwrap()
    }
}

pub struct IdArchive {
    pub id_key: SymK,
    pub ct: Vec<u8>,
}
