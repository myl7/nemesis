use async_trait::async_trait;
use prost::Message;
use rand::prelude::*;
use tonic::{Request, Response, Result, Status};
use uuid::Uuid;

use crate::crypto;
use crate::crypto::prelude::*;
use crate::db::{Db, IdArchive};
use crate::grpc::eems::eems_for_send_server::EemsForSend;
use crate::grpc::eems::{GenIdReq, GenIdRes, GenIdSignedPack};
use crate::msp::{EemsVec, Perm, SymEncPerm};
use crate::utils;

pub struct EemsForSendImpl {
    sk: SK,
    pub_src: Box<dyn PubSrc>,
    db: Db,
}

impl EemsForSendImpl {
    pub fn new(sk: SK, pub_src: Box<dyn PubSrc>, db: Db) -> Self {
        Self { sk, pub_src, db }
    }
}

#[async_trait]
impl EemsForSend for EemsForSendImpl {
    async fn gen_id(&self, req: Request<GenIdReq>) -> Result<Response<GenIdRes>> {
        let GenIdReq {
            ct,
            ct_hash,
            ct_hash_ct,
            auth: auth_opt,
        } = req.into_inner();

        let (user_id, user_id_sign) = {
            let auth = auth_opt.unwrap();
            let id = Uuid::from_slice(&auth.id).unwrap();
            let id_sign: Sign = auth.id_sign.try_into().unwrap();
            (id, id_sign)
        };
        let user_pk = self.pub_src.get(&user_id).unwrap().pk;
        if let Err(_) = crypto::pk_verify(&user_pk, user_id.as_bytes(), &user_id_sign) {
            return Err(Status::unauthenticated(""));
        }

        let (src_ct, sign, id_key) = self.gen_id_by_eems(&user_id, &ct_hash, ct_hash_ct);

        self.db
            .put_id_archive(&ct_hash.try_into().unwrap(), IdArchive { ct, id_key })
            .await;

        return Ok(Response::new(GenIdRes {
            src_ct,
            sign: sign.to_vec(),
        }));
    }
}

impl EemsForSendImpl {
    pub fn gen_id_by_eems(
        &self,
        user_id: &Uuid,
        ct_hash: &Vec<u8>,
        ct_hash_ct: Vec<u8>,
    ) -> (Vec<u8>, Sign, SymK) {
        let id_key: SymK = thread_rng().gen();
        let src_ct = crypto::sym_enc(&id_key, user_id.as_bytes());
        let signed_buf = GenIdSignedPack {
            src_ct: src_ct.clone(),
            ct_hash: ct_hash.clone(),
            ct_hash_ct,
        }
        .encode_to_vec();
        let sign = crypto::pk_sign(&self.sk, &signed_buf);
        (src_ct, sign, id_key)
    }
}

/// Source of public information of all entities
pub trait PubSrc: Sync + Send {
    fn get(&self, id: &Uuid) -> Option<PubInfo>;
}

pub struct PubInfo {
    pub pk: PK,
}

pub struct EemsIdShuffle<P: Perm> {
    shared_perm: P,
}

impl<P> EemsIdShuffle<P>
where
    P: Perm,
{
    pub fn new(shared_perm: P) -> Self {
        Self { shared_perm }
    }

    pub fn gen_shuffle_shares(
        &self,
        item_num: usize,
        mut ids: Vec<Vec<u8>>,
    ) -> ((EemsVec, EemsVec), Vec<Vec<u8>>) {
        let a1 = gen_rand_id_vec(item_num);
        let a2 = gen_rand_id_vec(item_num);
        let a3 = gen_rand_id_vec(item_num);
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

        let shuffle_share1 = EemsVec::OneVec(a1);
        let shuffle_share2 = EemsVec::ThreeVec(a2, a3, delta);

        ids.iter_mut().for_each(|item| {
            self.shared_perm.perm(item);
        });
        ids.iter_mut().for_each(|item| {
            eems_perm1.perm(item);
        });
        ids.iter_mut().for_each(|item| {
            eems_perm2.perm(item);
        });

        ((shuffle_share1, shuffle_share2), ids)
    }
}

fn gen_rand_id_vec(item_num: usize) -> Vec<Vec<u8>> {
    const ID_SIZE: usize = 164;
    let mut share = vec![vec![0u8; ID_SIZE]; item_num];
    share.iter_mut().for_each(|x| {
        rand::thread_rng().fill_bytes(x);
    });
    share
}
