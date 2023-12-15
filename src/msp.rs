use dcf::prg::Aes256HirosePrg as DcfAes256HirosePrg;
use dcf::{Dcf, DcfImpl, Share};
use dpf::prg::Aes256HirosePrg as DpfAes256HirosePrg;
use dpf::{Dpf, DpfImpl};
use group_math::int::U128Group;

use crate::crypto;
use crate::crypto::prelude::*;
use crate::utils;

#[derive(Clone)]
pub struct MspIdShuffle<P: Perm> {
    share: Vec<Vec<u8>>,
    msp_perm: P,
    eems_perm: P,
    eems_vec: EemsVec,
}

#[derive(Clone)]
pub enum EemsVec {
    OneVec(Vec<Vec<u8>>),
    ThreeVec(Vec<Vec<u8>>, Vec<Vec<u8>>, Vec<Vec<u8>>),
}

impl<P> MspIdShuffle<P>
where
    P: Perm,
{
    pub fn new(share: Vec<Vec<u8>>, msp_perm: P, eems_perm: P, eems_vec: EemsVec) -> Self {
        Self {
            share,
            msp_perm,
            eems_perm,
            eems_vec,
        }
    }

    pub fn run_msp_perm(&mut self) {
        self.share.iter_mut().for_each(|item| {
            self.msp_perm.perm(item);
        });
    }

    pub fn gen_shared_share(&self, z: Option<&Vec<Vec<u8>>>) -> Vec<Vec<u8>> {
        let mut result_share: Vec<Vec<u8>>;
        // a1, a2, a3, z1, z2, delta follow the naming in the paper
        match self.eems_vec {
            EemsVec::OneVec(ref a1) => {
                result_share = self.share.clone();
                let z2 = z.unwrap();
                result_share.iter_mut().enumerate().for_each(|(i, item)| {
                    utils::bytes_add(item, &z2[i]);
                    self.eems_perm.perm(item);
                    utils::bytes_minus(item, &a1[i]);
                });
            }
            EemsVec::ThreeVec(ref a2, ref a3, ref delta) => match z {
                Some(z1) => {
                    result_share = z1.clone();
                    result_share.iter_mut().enumerate().for_each(|(i, item)| {
                        self.eems_perm.perm(item);
                        utils::bytes_add(item, &delta[i]);
                        utils::bytes_add(item, &a2[i]);
                    });
                }
                None => {
                    result_share = self.share.clone();
                    result_share.iter_mut().enumerate().for_each(|(i, item)| {
                        utils::bytes_minus(item, &a3[i]);
                    });
                }
            },
        };
        result_share
    }
}

pub trait Perm: Clone {
    fn perm(&self, item: &mut Vec<u8>);
}

#[derive(Clone)]
pub struct SymEncPerm {
    pub key: SymK,
}

impl SymEncPerm {
    pub fn new(key: SymK) -> Self {
        Self { key }
    }
}

impl Perm for SymEncPerm {
    fn perm(&self, item: &mut Vec<u8>) {
        *item = crypto::sym_enc(&self.key, item);
    }
}

pub struct MspModeration {
    party: bool,
    dcf: DcfImpl<16, 16, DcfAes256HirosePrg<16, 1>>,
    dpf: DpfImpl<16, 16, DpfAes256HirosePrg<16, 1>>,
    id_hashes: Vec<Digest>,
}

impl MspModeration {
    pub fn new(
        party: bool,
        dcf_prg_key: &[u8; 32],
        dpf_prg_key: &[u8; 32],
        id_hashes: Vec<Digest>,
    ) -> Self {
        let dcf_prg = DcfAes256HirosePrg::new([dcf_prg_key]);
        let dcf = DcfImpl::new(dcf_prg);
        let dpf_prg = DpfAes256HirosePrg::new([dpf_prg_key]);
        let dpf = DpfImpl::new(dpf_prg);
        Self {
            party,
            dcf,
            dpf,
            id_hashes,
        }
    }

    pub fn sum_report(&self, shares: &[Share<16, U128Group>], values: &mut [u128]) {
        let mut buf = vec![U128Group(0); self.id_hashes.len()];
        shares.iter().for_each(|share| {
            self.dpf.eval(
                self.party,
                &share,
                &self.id_hashes.iter().collect::<Vec<_>>(),
                &mut buf.iter_mut().collect::<Vec<_>>(),
            );
            values.iter_mut().zip(buf.iter()).for_each(|(v, g)| {
                *v += g.0;
            });
        });
    }

    pub fn check_threhold(
        &self,
        kappa_shares: &[Share<16, U128Group>],
        gamma_shares: &[u128],
        values: &mut [u128],
    ) {
        gamma_shares
            .iter()
            .zip(values.iter_mut())
            .for_each(|(&gamma_share, value)| {
                *value = (*value).wrapping_add(gamma_share);
            });
        kappa_shares
            .iter()
            .zip(values.iter_mut())
            .for_each(|(kappa_share, value)| {
                let mut buf = U128Group(0);
                self.dcf.eval(
                    self.party,
                    &kappa_share,
                    &[&value.to_le_bytes()],
                    &mut [&mut buf],
                );
                *value = buf.0;
            });
    }
}
