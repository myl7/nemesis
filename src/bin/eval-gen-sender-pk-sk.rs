use rand::prelude::*;

use eemod::crypto;
use eemod::crypto::prelude::*;

fn main() {
    let sender_sk: SK = thread_rng().gen();
    let sender_sk_s = hex::encode(sender_sk);
    println!("sender_sk: {sender_sk_s}");
    let sender_pk = crypto::pk_pk(&sender_sk);
    let sender_pk_s = hex::encode(sender_pk);
    println!("sender_pk: {sender_pk_s}");
    println!("Set the envs as:");
    println!("export EEMOD_EVAL_SENDER_PK={sender_pk_s} EEMOD_EVAL_SENDER_SK={sender_sk_s}");
}
