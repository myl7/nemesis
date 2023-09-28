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
