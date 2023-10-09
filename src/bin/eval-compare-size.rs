use dcf::prg::Aes256HirosePrg;
use dcf::{BoundState, CmpFn, Dcf, DcfImpl};
use eemod::grpc::user::{KappaShare, KappaShareCw};
use group_math::int::U128Group;
use prost::Message;
use rand::prelude::*;

fn main() {
    let prg_key: [u8; 32] = thread_rng().gen();
    let prg = Aes256HirosePrg::new([&prg_key]);
    let dcf = DcfImpl::new(prg);
    let f = CmpFn::<16, 16, U128Group> {
        alpha: 10u128.to_le_bytes(),
        beta: U128Group(1),
    };

    let s00 = thread_rng().gen();
    let s01 = thread_rng().gen();
    let kappa_shares = {
        let share = dcf.gen(&f, [&s00, &s01], BoundState::GtBeta);
        let share0 = KappaShare {
            s0: share.s0s[0].to_vec(),
            cw_np1: share.cw_np1.0.to_le_bytes().to_vec(),
            cws: share
                .cws
                .into_iter()
                .map(|cw| KappaShareCw {
                    s: cw.s.to_vec(),
                    tl: cw.tl,
                    tr: cw.tr,
                    v: cw.v.0.to_le_bytes().to_vec(),
                })
                .collect(),
        };
        share0
    };
    let kappa_bs = kappa_shares.encode_to_vec();
    let kappa_size = kappa_bs.len();

    // Send them through a stream, so count the bytes one by one.
    // Same for the gammas.

    let total_item_size = kappa_size + 1; // sizeof u8
    print!("{total_item_size}B for every item")
}
