use anyhow::Result;
use double_ratchet::DoubleRatchet;
use double_ratchet_signal::signal::SignalCryptoProvider;
use prost::Message;
use rand::prelude::*;
// See the use
#[allow(deprecated)]
use rand_os::OsRng;
use tonic::transport::Channel;
use uuid::Uuid;

use crate::crypto;
use crate::crypto::prelude::*;
use crate::grpc::eems::eems_for_send_client::EemsForSendClient;
use crate::grpc::eems::{Auth, GenIdReq, GenIdRes};
use crate::grpc::user::{EeMsg, EeMsgHeader, Msg, MsgId};

pub struct Sender {
    id: Uuid,
    id_sign: Sign,
    eems_client: EemsForSendClient<Channel>,
}

impl Sender {
    pub fn new(id: Uuid, id_sign: Sign, eems_client: EemsForSendClient<Channel>) -> Self {
        Self {
            id,
            id_sign,
            eems_client,
        }
    }

    pub async fn gen_id(&self, body: &[u8], msg_key: SymK) -> Result<MsgId> {
        let ct = crypto::sym_enc(&msg_key, body);
        let ct_hash = crypto::hash(&ct);
        let ct_hash_ct = crypto::sym_enc(&msg_key, &ct_hash);

        let GenIdRes { src_ct, sign } = self
            .eems_client
            .clone()
            .gen_id(GenIdReq {
                ct,
                ct_hash: ct_hash.to_vec(),
                ct_hash_ct,
                auth: Some(Auth {
                    id: self.id.as_bytes().to_vec(),
                    id_sign: self.id_sign.to_vec(),
                }),
            })
            .await?
            .into_inner();

        Ok(MsgId {
            msg_key: msg_key.to_vec(),
            ct_hash: ct_hash.to_vec(),
            src_ct,
            sign,
        })
    }

    pub async fn gen_msg(
        &self,
        body: Vec<u8>,
        orig_id: Option<Vec<u8>>,
        shared_secret: [u8; 32],
        bob_pk: [u8; 32],
        asso_data: &[u8],
    ) -> Result<EeMsg> {
        let msg_key: SymK = thread_rng().gen();
        let msg_id = self.gen_id(&body, msg_key).await?;
        let id_bs = if let Some(orig_id_bs) = orig_id {
            orig_id_bs
        } else {
            msg_id.encode_to_vec()
        };
        let msg = Msg { id_bs, body };
        let msg_bs = msg.encode_to_vec();

        // FIXME: We need a newer Double Ratchet implementation
        #[allow(deprecated)]
        let mut os_rng = OsRng::new().unwrap();
        let mut alice = DoubleRatchet::<SignalCryptoProvider>::new_alice(
            &shared_secret.into(),
            bob_pk.into(),
            None,
            &mut os_rng,
        );
        let (msg_header, msg_ct) = alice.ratchet_encrypt(&msg_bs, asso_data, &mut os_rng);
        let ee_msg = EeMsg {
            header: Some(EeMsgHeader {
                n: msg_header.n,
                pn: msg_header.pn,
            }),
            ct: msg_ct,
        };

        Ok(ee_msg)
    }
}