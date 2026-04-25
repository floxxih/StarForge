use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use anyhow::{anyhow, Result};
use argon2::Argon2;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use dialoguer::Password;
use rand::RngCore;

pub fn prompt_password(prompt: &str, confirm: bool) -> Result<String> {
    let builder = Password::new()
        .with_prompt(prompt);

    let builder = if confirm {
        builder.with_confirmation("Confirm password", "Passwords mismatching")
    } else {
        builder
    };

    let pwd = builder.interact()?;
    if pwd.is_empty() {
        anyhow::bail!("Password cannot be empty");
    }
    Ok(pwd)
}

pub fn encrypt_secret(password: &str, secret: &str) -> Result<String> {
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    
    let argon2 = Argon2::default();
    let mut key = [0u8; 32];
    argon2.hash_password_into(password.as_bytes(), &salt, &mut key)
        .map_err(|e| anyhow!("Key derivation failed: {}", e))?;
        
    let cipher = Aes256Gcm::new(&key.into());
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, secret.as_bytes())
        .map_err(|e| anyhow!("Encryption failed: {}", e))?;
        
    let encoded_salt = BASE64.encode(salt);
    let encoded_nonce = BASE64.encode(nonce_bytes);
    let encoded_cipher = BASE64.encode(ciphertext);
    Ok(format!("{}:{}:{}", encoded_salt, encoded_nonce, encoded_cipher))
}

pub fn decrypt_secret(password: &str, bundle: &str) -> Result<String> {
    let parts: Vec<&str> = bundle.split(':').collect();
    if parts.len() != 3 {
        anyhow::bail!("Invalid encrypted bundle format");
    }
    
    let salt = BASE64.decode(parts[0])?;
    let nonce_bytes = BASE64.decode(parts[1])?;
    let ciphertext = BASE64.decode(parts[2])?;
    
    let argon2 = Argon2::default();
    let mut key = [0u8; 32];
    argon2.hash_password_into(password.as_bytes(), &salt, &mut key)
        .map_err(|e| anyhow!("Key derivation failed: {}", e))?;
        
    let cipher = Aes256Gcm::new(&key.into());
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let decrypted = cipher.decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| anyhow!("Decryption failed (incorrect password or corrupted data)"))?;
        
    String::from_utf8(decrypted).map_err(|e| anyhow!("Invalid UTF-8 in decrypted secret: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let password = "my_super_secret_password";
        let secret = "SXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX";
        
        let encrypted = encrypt_secret(password, secret).unwrap();
        assert_ne!(secret, encrypted);
        assert!(encrypted.contains(':'));
        
        // Correct password
        let decrypted = decrypt_secret(password, &encrypted).unwrap();
        assert_eq!(secret, decrypted);
        
        // Incorrect password
        let result = decrypt_secret("wrong_password", &encrypted);
        assert!(result.is_err());
    }
}
