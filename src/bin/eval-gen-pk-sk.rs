use rand::prelude::*;

use eemod::crypto;
use eemod::crypto::prelude::*;

fn main() {
    let sk: SK = thread_rng().gen();
    let sk_s = hex::encode(sk);
    println!("sk: {sk_s}");
    let pk = crypto::pk_pk(&sk);
    let pk_s = hex::encode(pk);
    println!("pk: {pk_s}");
    println!();
    println!("# Set the envs for the sender as:");
    println!("export EEMOD_EVAL_SENDER_PK={pk_s} EEMOD_EVAL_SENDER_SK={sk_s}");
    println!("# Set the envs for the EEMS as:");
    println!("export EEMOD_EVAL_EEMS_PK={pk_s} EEMOD_EVAL_EEMS_SK={sk_s}");
}
