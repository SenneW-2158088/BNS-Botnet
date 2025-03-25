use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use base64::{Engine as _, engine::general_purpose};

pub fn encrypt(text: &str, key_string: &str) -> Result<String, Box<dyn std::error::Error>> {
    let key_bytes = key_string.as_bytes();
    let key = &key_bytes[0..32];

    let cipher = Aes256Gcm::new_from_slice(key)?;

    let nonce_bytes = [0u8; 12];
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, text.as_bytes())?;

    Ok(general_purpose::STANDARD.encode(ciphertext))
}

pub fn decrypt(text: &str, key_string: &str) -> Result<String, Box<dyn std::error::Error>> {
    let key_bytes = key_string.as_bytes();
    let key = &key_bytes[0..32];

    let cipher = Aes256Gcm::new_from_slice(key)?;

    let nonce_bytes = [0u8; 12];
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = general_purpose::STANDARD.decode(text)?;
    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())?;

    Ok(String::from_utf8(plaintext)?)
}
