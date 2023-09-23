use std::env::var;

use prost::Message;
use rand::prelude::*;
use uuid::Uuid;

use eemod::crypto;
use eemod::crypto::prelude::*;
use eemod::grpc::eems::eems_for_send_client::EemsForSendClient;
use eemod::user::Sender;

#[tokio::main]
async fn main() {
    let sender_id_s = "f7584d00-6948-4e9c-b444-ff757f4dd9c1";
    let sender_id = Uuid::parse_str(sender_id_s).unwrap();
    let sender_sk: SK = hex::decode(var("EEMOD_EVAL_SENDER_SK").unwrap())
        .unwrap()
        .try_into()
        .unwrap();
    let sender_id_sign = crypto::pk_sign(&sender_sk, sender_id.as_bytes());
    let eems_addr = "127.0.0.1:8000";
    let eems_url = format!("http://{eems_addr}");
    let eems_client = EemsForSendClient::connect(eems_url).await.unwrap();

    let sender = Sender::new(sender_id, sender_id_sign, eems_client);
    let body = b"Hello, world!";
    let msg_key: SymK = thread_rng().gen();
    let msg_id = sender.gen_id(body, msg_key).await.unwrap();
    let msg_id_bs = msg_id.encode_to_vec();
    let msg_id_size = msg_id_bs.len();

    println!("msg_id size: {msg_id_size}")
}
