use anyhow::{Context, Result, bail};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use chacha20poly1305::{
    KeyInit, XChaCha20Poly1305, XNonce,
    aead::{Aead, Payload},
};
use zeroize::Zeroizing;

use crate::KeyEnvelope;

const ROOT_AAD: &[u8] = b"pensive/vault-root-envelope/1";

pub fn random_bytes<const N: usize>() -> Result<[u8; N]> {
    let mut value = [0_u8; N];
    getrandom::fill(&mut value)
        .map_err(|error| anyhow::anyhow!("operating system random generator failed: {error}"))?;
    Ok(value)
}

pub fn derive_passphrase_key(passphrase: &str, salt: &[u8]) -> Result<Zeroizing<[u8; 32]>> {
    let params = Params::new(65_536, 3, 1, Some(32))
        .map_err(|error| anyhow::anyhow!("invalid Argon2 parameters: {error}"))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut output = Zeroizing::new([0_u8; 32]);
    argon
        .hash_password_into(passphrase.as_bytes(), salt, output.as_mut())
        .map_err(|error| anyhow::anyhow!("Argon2id key derivation failed: {error}"))?;
    Ok(output)
}

pub fn derive_domain_key(root: &[u8; 32], domain: &str) -> Zeroizing<[u8; 32]> {
    Zeroizing::new(blake3::derive_key(domain, root))
}

pub fn wrap_root_key(passphrase: &str, root_key: &[u8; 32]) -> Result<KeyEnvelope> {
    let salt = random_bytes::<16>()?;
    let nonce = random_bytes::<24>()?;
    let wrapping_key = derive_passphrase_key(passphrase, &salt)?;
    let cipher =
        XChaCha20Poly1305::new_from_slice(wrapping_key.as_ref()).context("invalid wrapping key")?;
    let ciphertext = cipher
        .encrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: root_key,
                aad: ROOT_AAD,
            },
        )
        .map_err(|_| anyhow::anyhow!("vault root encryption failed"))?;
    Ok(KeyEnvelope {
        protocol: "pensive-key-envelope/1".into(),
        kdf: "argon2id:m=65536,t=3,p=1".into(),
        salt: B64.encode(salt),
        nonce: B64.encode(nonce),
        ciphertext: B64.encode(ciphertext),
    })
}

pub fn unwrap_root_key(passphrase: &str, envelope: &KeyEnvelope) -> Result<Zeroizing<[u8; 32]>> {
    if envelope.protocol != "pensive-key-envelope/1" {
        bail!("unsupported key envelope protocol")
    }
    let salt = B64
        .decode(&envelope.salt)
        .context("invalid envelope salt")?;
    let nonce = B64
        .decode(&envelope.nonce)
        .context("invalid envelope nonce")?;
    let ciphertext = B64
        .decode(&envelope.ciphertext)
        .context("invalid envelope ciphertext")?;
    if nonce.len() != 24 {
        bail!("invalid envelope nonce length")
    }
    let wrapping_key = derive_passphrase_key(passphrase, &salt)?;
    let cipher =
        XChaCha20Poly1305::new_from_slice(wrapping_key.as_ref()).context("invalid wrapping key")?;
    let plaintext = cipher
        .decrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: &ciphertext,
                aad: ROOT_AAD,
            },
        )
        .map_err(|_| anyhow::anyhow!("wrong passphrase or tampered key envelope"))?;
    let key: [u8; 32] = plaintext
        .try_into()
        .map_err(|_| anyhow::anyhow!("invalid vault root key length"))?;
    Ok(Zeroizing::new(key))
}

pub fn seal(key: &[u8; 32], aad: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
    let nonce = random_bytes::<24>()?;
    let cipher = XChaCha20Poly1305::new_from_slice(key).context("invalid encryption key")?;
    let ciphertext = cipher
        .encrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|_| anyhow::anyhow!("encryption failed"))?;
    let mut sealed = Vec::with_capacity(4 + nonce.len() + ciphertext.len());
    sealed.extend_from_slice(b"PMO1");
    sealed.extend_from_slice(&nonce);
    sealed.extend_from_slice(&ciphertext);
    Ok(sealed)
}

pub fn open(key: &[u8; 32], aad: &[u8], sealed: &[u8]) -> Result<Vec<u8>> {
    if sealed.len() < 4 + 24 + 16 || &sealed[..4] != b"PMO1" {
        bail!("invalid encrypted object")
    }
    let (nonce, ciphertext) = sealed[4..].split_at(24);
    let cipher = XChaCha20Poly1305::new_from_slice(key).context("invalid encryption key")?;
    cipher
        .decrypt(
            XNonce::from_slice(nonce),
            Payload {
                msg: ciphertext,
                aad,
            },
        )
        .map_err(|_| anyhow::anyhow!("wrong key or tampered encrypted object"))
}

pub fn b64(bytes: impl AsRef<[u8]>) -> String {
    B64.encode(bytes)
}

pub fn decode_b64(value: &str) -> Result<Vec<u8>> {
    B64.decode(value).context("invalid base64")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seal_roundtrip_and_tamper_rejection() {
        let key = random_bytes::<32>().expect("key");
        let sealed = seal(&key, b"source/1", b"private memory").expect("seal");
        assert_eq!(
            open(&key, b"source/1", &sealed).expect("open"),
            b"private memory"
        );

        let mut tampered = sealed;
        let last = tampered.len() - 1;
        tampered[last] ^= 1;
        assert!(open(&key, b"source/1", &tampered).is_err());
    }

    #[test]
    fn wrong_passphrase_rejected() {
        let root = random_bytes::<32>().expect("root");
        let envelope = wrap_root_key("correct horse", &root).expect("wrap");
        assert_eq!(
            unwrap_root_key("correct horse", &envelope)
                .expect("unwrap")
                .as_ref(),
            &root
        );
        assert!(unwrap_root_key("wrong", &envelope).is_err());
    }

    #[test]
    fn domain_keys_are_distinct() {
        let root = [7_u8; 32];
        assert_ne!(
            derive_domain_key(&root, "pensive/database-key/v1").as_ref(),
            derive_domain_key(&root, "pensive/object-key/v1").as_ref()
        );
    }
}
