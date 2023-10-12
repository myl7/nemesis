use std::env::var;

use criterion::{criterion_group, criterion_main, Criterion};
use double_ratchet::KeyPair as DrKeyPair;
use double_ratchet_signal::signal::KeyPair;
use rand::prelude::*;
use tokio::runtime::Builder;
use uuid::Uuid;
// See the use
#[allow(deprecated)]
use rand_os::OsRng;

use eemod::crypto;
use eemod::crypto::prelude::*;
use eemod::grpc::eems::eems_for_send_client::EemsForSendClient;
use eemod::user::{Receiver, Sender};

fn bench(c: &mut Criterion) {
    let rt = Builder::new_multi_thread().enable_all().build().unwrap();

    let sender_id_s = "f7584d00-6948-4e9c-b444-ff757f4dd9c1";
    let sender_id = Uuid::parse_str(sender_id_s).unwrap();
    let sender_sk: SK = hex::decode(var("EEMOD_EVAL_SENDER_SK").unwrap())
        .unwrap()
        .try_into()
        .unwrap();
    let sender_pk: PK = crypto::pk_pk(&sender_sk);
    let sender_id_sign = crypto::pk_sign(&sender_sk, sender_id.as_bytes());
    let eems_addr = var("EEMOD_EVAL_EEMS_ADDR").unwrap();
    let eems_url = format!("http://{eems_addr}");
    let eems_client = rt.block_on(async { EemsForSendClient::connect(eems_url).await.unwrap() });
    let sender = Sender::new(sender_id, sender_id_sign, eems_client);

    let body_size = 1000; // 1KB same as Peale
    let mut body = vec![0; body_size];
    thread_rng().fill_bytes(&mut body);
    let shared_secret: [u8; 32] = thread_rng().gen();
    let receiver_sk: SK = hex::decode(var("EEMOD_EVAL_RECEIVER_SK").unwrap())
        .unwrap()
        .try_into()
        .unwrap();
    let receiver_pk: PK = crypto::pk_pk(&receiver_sk);
    // FIXME: We need a newer Double Ratchet implementation
    #[allow(deprecated)]
    let mut os_rng = OsRng::new().unwrap();
    let kp = KeyPair::new(&mut os_rng);
    let kp_pk = kp.public();
    let asso_data = b"A2B:SessionID=42";

    c.bench_function("gen_msg", |b| {
        b.to_async(&rt)
            .iter(|| sender.gen_msg(body.clone(), None, shared_secret, receiver_pk, asso_data));
    });

    let eemsg = rt.block_on(async {
        sender
            .gen_msg(
                body.clone(),
                None,
                shared_secret,
                kp_pk.as_ref().try_into().unwrap(),
                asso_data,
            )
            .await
            .unwrap()
    });

    let eems_pk: PK = hex::decode(var("EEMOD_EVAL_EEMS_PK").unwrap())
        .unwrap()
        .try_into()
        .unwrap();
    let receiver = Receiver::new(eems_pk);

    c.bench_function("recv_msg", |b| {
        b.to_async(&rt).iter(|| {
            receiver.recv_msg(
                eemsg.clone(),
                shared_secret,
                sender_pk,
                asso_data,
                kp.clone(),
            )
        });
    });
}

// Since we will communicate with the remote EEMS
criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench
}
criterion_main!(benches);
