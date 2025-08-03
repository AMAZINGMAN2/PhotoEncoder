use image::{ImageReader, DynamicImage, GenericImage, GenericImageView, Rgba};
use image::codecs::png::PngEncoder;
use image::ImageEncoder;
use image::ColorType;
use std::io::Cursor;

pub fn encode_image(img_bytes: &[u8], secret_bytes: &[u8]) -> Result<Vec<u8>, String> {
    // Load image from bytes and convert to RGBA8 immediately
    let img = ImageReader::new(Cursor::new(img_bytes))
        .with_guessed_format()
        .map_err(|e| format!("Image format guess error: {}", e))?
        .decode()
        .map_err(|e| format!("Image decode error: {}", e))?
        .to_rgba8(); // Convert to RGBA8 immediately

    let (width, height) = img.dimensions();
    let max_capacity = (width * height * 4) / 8;
    println!("Image: {}x{}, Capacity: {} bytes, Secret: {} bytes", width, height, max_capacity, secret_bytes.len());

    if secret_bytes.len() > max_capacity as usize {
        return Err("Secret data too large to encode in image".to_string());
    }

    // Encode secret size in first 32 bits
    let secret_len = secret_bytes.len() as u32;
    println!("Encoding secret length: {} bytes", secret_len);
    println!("Secret length as binary: {:032b}", secret_len);
    
    let mut secret_bits = secret_len.to_be_bytes()
        .iter()
        .flat_map(|byte| (0..8).rev().map(move |i| (byte >> i) & 1))
        .collect::<Vec<u8>>();
    
    println!("First 32 bits being encoded: {:?}", &secret_bits[0..32]);

    // Then encode secret data bits
    for &byte in secret_bytes {
        secret_bits.extend((0..8).rev().map(|i| (byte >> i) & 1));
    }

    // Work directly with the raw pixel data
    let mut img_data = img.into_raw();
    
    // Modify LSB of each byte in the raw data
    for (i, &bit) in secret_bits.iter().enumerate() {
        if i >= img_data.len() {
            break;
        }
        img_data[i] = (img_data[i] & 0xFE) | bit;
    }

    // Create a new image from the modified raw data
    let modified_img = image::RgbaImage::from_raw(width, height, img_data)
        .ok_or("Failed to create image from raw data")?;

    // Save image as PNG to bytes vec
    let mut cursor = Cursor::new(Vec::new());
    let encoder = PngEncoder::new(&mut cursor);
    encoder
        .write_image(
            modified_img.as_raw(),
            width,
            height,
            ColorType::Rgba8.into(),
        )
        .map_err(|e| format!("Image encode error: {}", e))?;

    Ok(cursor.into_inner())
}

pub fn decode_image(img_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let img = ImageReader::new(Cursor::new(img_bytes))
        .with_guessed_format()
        .map_err(|e| format!("Image format guess error: {}", e))?
        .decode()
        .map_err(|e| format!("Image decode error: {}", e))?;

    let (width, height) = img.dimensions();

    let mut bits = Vec::new();

    // Read the first 32 bits = secret length
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

    // Convert first 32 bits to secret length in bytes - WITH DEBUGGING
    println!("First 32 bits: {:?}", &bits[0..32]);
    
    let mut secret_len = 0u32;
    for i in 0..32 {
        secret_len = (secret_len << 1) | (bits[i] as u32);
    }
    let secret_len = secret_len as usize;
    
    println!("Decoded secret length: {} bytes", secret_len);
    println!("Secret length as binary: {:032b}", secret_len as u32);

    let total_bits = secret_len * 8;
    let mut secret_bits = Vec::with_capacity(total_bits);
    let mut bit_collected = 0;
    let mut skip = 32; // Skip the first 32 bits already read

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

    // Convert bits to bytes - CORRECTED VERSION
    let mut secret = Vec::with_capacity(secret_len);
    for i in 0..secret_len {
        let mut byte = 0u8;
        for bit in 0..8 {
            byte = (byte << 1) | secret_bits[i * 8 + bit];
        }
        secret.push(byte);
    }

    Ok(secret)
}
