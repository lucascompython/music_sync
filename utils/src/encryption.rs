use aes::{
    cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit},
    Aes128,
};
use block_padding::Pkcs7;
use rand::Rng;
use sha2::{Digest, Sha256};

type Aes128CbcEnc = cbc::Encryptor<Aes128>;
type Aes128CbcDec = cbc::Decryptor<Aes128>;

pub struct TokenVerifier {
    key: [u8; 16],
    iv: [u8; 16],
}

impl TokenVerifier {
    pub fn new(token: &str) -> Self {
        let mut rng = rand::thread_rng();
        let mut iv = [0u8; 16];
        rng.fill(&mut iv);

        let key = Self::derive_key_from_token(token);

        Self { key, iv }
    }

    pub fn derive_key_from_token(token: &str) -> [u8; 16] {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let result = hasher.finalize();
        let mut key = [0u8; 16];
        key.copy_from_slice(&result[..16]);
        key
    }

    pub fn encrypt(&self, data: &[u8]) -> String {
        let mut buf = vec![0u8; data.len() + 16];

        let ct = Aes128CbcEnc::new(&self.key.into(), &self.iv.into())
            .encrypt_padded_b2b_mut::<Pkcs7>(data, &mut buf)
            .unwrap();

        let mut result = Vec::with_capacity(self.iv.len() + ct.len());
        result.extend_from_slice(&self.iv);
        result.extend_from_slice(ct);

        hex::encode(&result)
    }

    pub fn decrypt(&self, data: &str) -> Option<String> {
        let data = hex::decode(data).unwrap();

        let (iv, ct) = data.split_at(16);

        let mut iv_array = [0u8; 16];
        iv_array.copy_from_slice(iv);

        let mut buf = vec![0u8; ct.len()];
        match Aes128CbcDec::new(&self.key.into(), &iv_array.into())
            .decrypt_padded_b2b_mut::<Pkcs7>(ct, &mut buf)
        {
            Ok(_) => {
                let buf_len = buf.len();
                let padding = buf[buf_len - 1] as usize;
                buf.truncate(buf_len - padding);

                // Ok(String::from_utf8(buf)?)
                String::from_utf8(buf).ok()
            }
            Err(_) => None,
        }
    }

    pub fn verify(&self, token: &str) -> bool {
        let key = Self::derive_key_from_token(token);
        self.key == key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify() {
        let token = "0123456789abcdef0123456789abcdef";
        let verifier = TokenVerifier::new(token);
        let encrypted_token = verifier.encrypt(token.as_bytes());
        let decrypted_token = verifier.decrypt(&encrypted_token).unwrap();
        assert_eq!(token, decrypted_token);
        assert!(verifier.verify(&decrypted_token));
    }

    #[test]
    fn test_verify_fail() {
        let token = "0123456789abcdef0123456789abcdef";
        let verifier = TokenVerifier::new(token);
        let encrypted_token = verifier.encrypt(token.as_bytes());
        let decrypted_token = verifier.decrypt(&encrypted_token).unwrap();

        // Verify with the correct token and IV
        assert!(verifier.verify(&decrypted_token));

        // Verify with a modified token
        let wrong_token = "wrong_token";
        assert!(!verifier.verify(&wrong_token));
    }
}
