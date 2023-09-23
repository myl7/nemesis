//! Basic cryptographic primitives
//!
//! ALL dependencies come from [Rust Crypto] with strong security
//!
//! [Rust Crypto]: https://github.com/RustCrypto

pub mod prelude {
    pub use super::hash::Digest;
    pub use super::pk::{Sign, PK, SK};
    pub use super::sym::SymK;
}

pub use hash::hash;
pub mod hash {
    use sha2::Sha256;

    pub type Digest = [u8; 32];

    pub fn hash(data: &[u8]) -> Digest {
        use sha2::Digest;
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }
}

pub use sym::{sym_dec, sym_enc};
pub mod sym {
    //! `sym` = symmetric, `K` = key, `pt` = plaintext, `ct` = ciphertext

    use aes_gcm::aead::Aead;
    use aes_gcm::{Aes128Gcm, Key, KeyInit};

    pub type SymK = [u8; 16 + 12];

    pub fn sym_enc(key: &SymK, pt: &[u8]) -> Vec<u8> {
        let key_obj = Key::<Aes128Gcm>::from_slice(&key[..16]);
        let nonce = (&key[16..]).into();
        let cipher = Aes128Gcm::new(key_obj);
        cipher.encrypt(nonce, pt).unwrap()
    }

    pub fn sym_dec(key: &SymK, ct: &[u8]) -> Result<Vec<u8>, ()> {
        let key_obj = Key::<Aes128Gcm>::from_slice(&key[..16]);
        let nonce = (&key[16..]).into();
        let cipher = Aes128Gcm::new(key_obj);
        cipher.decrypt(nonce, ct).map_err(|_| ())
    }

    #[cfg(test)]
    mod tests {
        use rand::prelude::*;

        use super::*;

        #[test]
        fn test_sym_enc_dec_ok() {
            let key: SymK = thread_rng().gen();
            let pt = b"Hello, world!";
            let ct = sym_enc(&key, pt);
            assert_eq!(sym_dec(&key, &ct).unwrap(), pt);
        }
    }
}

pub use pk::{pk_pk, pk_sign, pk_verify};
pub mod pk {
    //! `kp` = key pair, `sk` = secret key (signing key), `pk` = public key (verifying key), `sign` = signature

    use ring_compat::signature::ed25519::{Signature, SigningKey, VerifyingKey};
    use ring_compat::signature::{Signer, Verifier};

    pub type SK = [u8; 32];
    pub type PK = [u8; 32];
    pub type Sign = [u8; 64];

    /// Get `pk` from randomly sampled `sk`
    pub fn pk_pk(sk: &SK) -> PK {
        let sk_obj = SigningKey::from_slice(sk).unwrap();
        sk_obj.verifying_key().0
    }

    pub fn pk_sign(sk: &SK, data: &[u8]) -> Sign {
        let sk_obj = SigningKey::from_slice(sk).unwrap();
        let sign = sk_obj.sign(data);
        sign.into()
    }

    pub fn pk_verify(pk: &PK, data: &[u8], sign: &Sign) -> Result<(), ()> {
        let pk_obj = VerifyingKey::from_slice(pk).unwrap();
        let sign_obj = Signature::from_slice(sign).unwrap();
        pk_obj.verify(data, &sign_obj).map_err(|_| ())
    }

    #[cfg(test)]
    mod tests {
        use rand::prelude::*;

        use super::*;

        #[test]
        fn test_sign_verify_ok() {
            let sk: SK = thread_rng().gen();
            let pk = pk_pk(&sk);
            let data = b"Hello, world!";
            let sign = pk_sign(&sk, data);
            assert!(pk_verify(&pk, data, &sign).is_ok());
        }
    }
}
