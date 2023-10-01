use prost::Message;
use rand::prelude::*;

use eemod::user::Reporter;

fn main() {
    let prg_key: [u8; 32] = thread_rng().gen();
    let reporter = Reporter::new(&prg_key);
    // Since `id` is hashed, random `id` can still work for this evaluation
    const ID: &[u8] = b"Hello, world!";
    let s0s: [[u8; 16]; 2] = thread_rng().gen();
    let shares = reporter.gen_id_shares(ID, [&s0s[0], &s0s[1]]);
    let share_bs = [shares[0].encode_to_vec(), shares[1].encode_to_vec()];
    let share_size = share_bs[0].len() + share_bs[1].len();
    println!("2 share total size: {share_size}")
}
