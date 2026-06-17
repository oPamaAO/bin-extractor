//! Decrypt Albion Online .bin game data files.
//!
//! Each `.bin` file is:
//!   DES-CBC(key, iv) → gzip(data) → XML (UTF-8)
//!
//! DES parameters (reversed from game client):
//!   Key: 30 EF 72 47 42 F2 04 32
//!   IV:  0E A6 DC 89 DB ED DC 4F

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use std::io::Read;

const DES_KEY: [u8; 8] = [0x30, 0xEF, 0x72, 0x47, 0x42, 0xF2, 0x04, 0x32];
const DES_IV: [u8; 8] = [0x0E, 0xA6, 0xDC, 0x89, 0xDB, 0xED, 0xDC, 0x4F];

/// Decrypt a `.bin` file → XML string.
pub fn decrypt_bin(data: &[u8]) -> Result<String> {
    let decrypted = decrypt_des_cbc(data)?;
    gzip_decompress(&decrypted)
}

// ── DES-CBC ─────────────────────────────────────────────────────────

fn des_ecb_decrypt_block(key: &[u8; 8], block: &[u8; 8]) -> [u8; 8] {
    use des::cipher::generic_array::GenericArray;
    use des::cipher::{BlockDecrypt, KeyInit};
    use des::Des;

    let cipher = Des::new_from_slice(key).expect("DES key must be 8 bytes");
    let mut buf = GenericArray::clone_from_slice(block);
    cipher.decrypt_block(&mut buf);
    buf.into()
}

/// Manual DES-CBC: P_i = DES_DECRYPT(K, C_i) XOR C_{i-1}, C_0 = IV
fn decrypt_des_cbc(data: &[u8]) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Ok(Vec::new());
    }
    if data.len() % 8 != 0 {
        anyhow::bail!(
            "Input length {} is not a multiple of 8 (DES block size)",
            data.len()
        );
    }

    let mut result = Vec::with_capacity(data.len());
    let mut prev = DES_IV;

    for chunk in data.chunks(8) {
        let block: &[u8; 8] = chunk.try_into().unwrap();
        let decrypted = des_ecb_decrypt_block(&DES_KEY, block);

        for i in 0..8 {
            result.push(decrypted[i] ^ prev[i]);
        }
        prev = *block;
    }

    // Strip PKCS7 padding
    if let Some(&last) = result.last() {
        let pad = last as usize;
        if pad > 0 && pad <= 8 && pad <= result.len() {
            result.truncate(result.len() - pad);
        }
    }

    Ok(result)
}

// ── GZip ────────────────────────────────────────────────────────────

fn gzip_decompress(data: &[u8]) -> Result<String> {
    let mut decoder = GzDecoder::new(data);
    let mut s = String::new();
    decoder
        .read_to_string(&mut s)
        .context("Failed to gzip decompress (not valid gzip data)")?;
    Ok(s)
}
