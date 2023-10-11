use eemod::grpc::user::IdShareWithMac;
use prost::Message;
use rand::prelude::*;

use eemod::crypto::prelude::*;
use eemod::user::{Reporter, ReporterGenMac};

fn main() {
    let prg_key: [u8; 32] = thread_rng().gen();
    let reporter = Reporter::new(&prg_key);
    // Since `id` is hashed, random `id` can still work for this evaluation
    const ID: &[u8] = b"Hello, world!";
    let s0s: [[u8; 16]; 2] = thread_rng().gen();
    let shares = reporter.gen_id_shares(ID, [&s0s[0], &s0s[1]]);

    let mk: SymK = thread_rng().gen();
    let report_gen = ReporterGenMac::new();
    let (gamma, tao0, _tao1) = report_gen.gen(&mk);

    let share_mac = IdShareWithMac {
        share: Some(shares[0].clone()),
        mac_gamma: gamma,
        mac_tao: tao0,
    };
    let share_mac_bs = share_mac.encode_to_vec();
    let share_mac_size = share_mac_bs.len() * 2;
    println!("2 share total size: {share_mac_size}")
}
