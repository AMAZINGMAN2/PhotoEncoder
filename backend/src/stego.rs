use aes_gcm::{Nonce}; // AES-GCM 256-bit
use aes_gcm::{Aes256Gcm, KeyInit, aead::Aead};
use aes_gcm::aead::generic_array::GenericArray;
use rand::RngCore;
use sha2::{Sha256, Digest};
use image::{ImageReader, GenericImageView, ImageEncoder, ColorType};
use image::codecs::png::PngEncoder;
use std::io::Cursor;

fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

fn encrypt_secret(secret: &[u8], password: &[u8]) -> Result<Vec<u8>, String> {

    let hash = sha256_hash(password);
    let key = GenericArray::from_slice(&hash);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, secret)
        .map_err(|e| format!("Encryption error: {:?}", e))?;

    let mut encrypted_data = nonce_bytes.to_vec();
    encrypted_data.extend(ciphertext);

    Ok(encrypted_data)
}

fn decrypt_secret(encrypted_data: &[u8], password: &[u8]) -> Result<Vec<u8>, String> {
    if encrypted_data.len() < 12 {
        return Err("Encrypted data too short".into());
    }
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);


    let hash = sha256_hash(password);
    let key = GenericArray::from_slice(&hash);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption error: {:?}", e))
}

pub fn encode_image(img_bytes: &[u8], secret_bytes: &[u8], password: Option<&[u8]>) -> Result<Vec<u8>, String> {
    let secret_to_encode = if let Some(pw) = password {
        encrypt_secret(secret_bytes, pw)?
    } else {
        secret_bytes.to_vec()
    };

    // The rest is the same as your existing encode_image logic,
    // just replace `secret_bytes` with `secret_to_encode`
    
    let img = ImageReader::new(Cursor::new(img_bytes))
        .with_guessed_format()
        .map_err(|e| format!("Image format guess error: {}", e))?
        .decode()
        .map_err(|e| format!("Image decode error: {}", e))?
        .to_rgba8();

    let (width, height) = img.dimensions();
    let max_capacity = (width * height * 4) / 8;

    if secret_to_encode.len() > max_capacity as usize {
        return Err("Secret data too large to encode in image".to_string());
    }

    let secret_len = secret_to_encode.len() as u32;

    let mut secret_bits = secret_len.to_be_bytes()
        .iter()
        .flat_map(|byte| (0..8).rev().map(move |i| (byte >> i) & 1))
        .collect::<Vec<u8>>();

    for &byte in &secret_to_encode {
        secret_bits.extend((0..8).rev().map(|i| (byte >> i) & 1));
    }

    let mut img_data = img.into_raw();

    for (i, &bit) in secret_bits.iter().enumerate() {
        if i >= img_data.len() {
            break;
        }
        img_data[i] = (img_data[i] & 0xFE) | bit;
    }

    let modified_img = image::RgbaImage::from_raw(width, height, img_data)
        .ok_or("Failed to create image from raw data")?;

    let mut cursor = Cursor::new(Vec::new());
    let encoder = PngEncoder::new(&mut cursor);
    encoder.write_image(modified_img.as_raw(), width, height, ColorType::Rgba8.into())
        .map_err(|e| format!("Image encode error: {}", e))?;

    Ok(cursor.into_inner())
}

pub fn decode_image(img_bytes: &[u8], password: Option<&[u8]>) -> Result<Vec<u8>, String> {
    let img = ImageReader::new(Cursor::new(img_bytes))
        .with_guessed_format()
        .map_err(|e| format!("Image format guess error: {}", e))?
        .decode()
        .map_err(|e| format!("Image decode error: {}", e))?;

    let (width, height) = img.dimensions();

    let mut bits = Vec::new();

    let mut count = 0;
    'outer: for y in 0..height {
        for x in 0..width {
            let px = img.get_pixel(x, y);
            for ch in 0..4 {
                bits.push(px[ch] & 1);
                count += 1;
                if count == 32 {
                    break 'outer;
                }
            }
        }
    }

    if bits.len() < 32 {
        return Err("Image too small or corrupted".to_string());
    }

    let mut secret_len = 0u32;
    for i in 0..32 {
        secret_len = (secret_len << 1) | (bits[i] as u32);
    }
    let secret_len = secret_len as usize;

    let total_bits = secret_len * 8;
    let mut secret_bits = Vec::with_capacity(total_bits);
    let mut bit_collected = 0;
    let mut skip = 32;

    'outer2: for y in 0..height {
        for x in 0..width {
            let px = img.get_pixel(x, y);
            for ch in 0..4 {
                if skip > 0 {
                    skip -= 1;
                    continue;
                }
                if bit_collected == total_bits {
                    break 'outer2;
                }
                secret_bits.push(px[ch] & 1);
                bit_collected += 1;
            }
        }
    }

    if secret_bits.len() < total_bits {
        return Err(format!(
            "Image does not contain enough data: expected {}, got {}",
            total_bits,
            secret_bits.len()
        ));
    }

    let mut secret = Vec::with_capacity(secret_len);
    for i in 0..secret_len {
        let mut byte = 0u8;
        for bit in 0..8 {
            byte = (byte << 1) | secret_bits[i * 8 + bit];
        }
        secret.push(byte);
    }

    // If password provided, decrypt; else return raw secret
    if let Some(pw) = password {
        decrypt_secret(&secret, pw)
    } else {
        Ok(secret)
    }
}
