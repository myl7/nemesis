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

        let id_key: SymK = thread_rng().gen();
        let src_ct = crypto::sym_enc(&id_key, user_id.as_bytes());
        let signed_buf = GenIdSignedPack {
            src_ct: src_ct.clone(),
            ct_hash: ct_hash.clone(),
            ct_hash_ct,
        }
        .encode_to_vec();
        let sign = crypto::pk_sign(&self.sk, &signed_buf);

        self.db
            .put_id_archive(&ct_hash.try_into().unwrap(), IdArchive { ct, id_key })
            .await;

        return Ok(Response::new(GenIdRes {
            src_ct,
            sign: sign.to_vec(),
        }));
    }
}

/// Source of public information of all entities
pub trait PubSrc: Sync + Send {
    fn get(&self, id: &Uuid) -> Option<PubInfo>;
}

pub struct PubInfo {
    pub pk: PK,
}
