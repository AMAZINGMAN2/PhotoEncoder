# PhotoEncoder — Rust Steganography Tool with AES-GCM Encryption

PhotoEncoder is a Rust-based steganography tool that hides secret data inside PNG images using **Least Significant Bit (LSB)** encoding. It also supports **optional AES-256-GCM encryption** to securely protect the embedded data.

---

## 🔒 Features

- 🖼️ Embed secret data into images using LSB steganography
- 🔐 Optional AES-256-GCM encryption with password protection
- 📦 Compact output — stores data inside PNG images
- ⚙️ Automatic decryption detection via a magic header (`ENC!`)

---

## 🛠️ How It Works

### 1. 🧠 Least Significant Bit (LSB) Encoding

Digital images are composed of pixels, and each pixel has **channels** like Red, Green, Blue, and optionally Alpha. Each of these channels is stored as an 8-bit value (0–255).

To hide information, we **modify the least significant bit** (LSB) of each byte. For example:

| Original Byte | Binary | Modified Bit | Result |
|---------------|--------|--------------|--------|
| `10110110`    | `10110110` | 1 → 0   | `10110110` → `10110110` |
| `10110111`    | `10110111` | 1 → 0   | `10110111` → `10110110` |

This small change doesn't significantly affect the visual appearance of the image but lets us store binary data.

---

### 2. 🧪 Encoding Process

1. Load the input PNG image and convert it to RGBA.
2. Optionally encrypt the secret data using AES-GCM if a password is provided.
3. Prefix the encrypted data with a 4-byte magic header `ENC!` (if encrypted).
4. Encode the **length of the secret data (4 bytes, big-endian)** into the first 32 LSBs of the image.
5. Encode each bit of the data into the remaining image LSBs.

---

### 3. 🔓 Decoding Process

1. Read the first 32 bits to determine the length of the secret.
2. Extract `secret_len * 8` LSBs to reconstruct the hidden data.
3. If the result starts with `ENC!`:
   - Require a password.
   - Use AES-GCM to decrypt the message using:
     - SHA-256 hash of the password as the key
     - First 12 bytes after `ENC!` as the nonce
     - Remaining bytes as the ciphertext
4. Return the raw or decrypted message.

---

### 4. 🔐 AES-GCM Encryption

- Uses [`aes-gcm`](https://docs.rs/aes-gcm) crate with 256-bit keys.
- Password is hashed with SHA-256 to derive a 32-byte AES key.
- AES-GCM provides **authenticated encryption**, ensuring that tampering can be detected.

---


## 🚀 Example Usage

### 🖥️ Run Locally

You can run PhotoEncoder locally using both the frontend and backend. The backend has a Docker setup, or you can run both parts manually:

---

#### 🐳 Backend Only: Using Docker

First clone this repository:
```bash
git clone https://github.com/AMAZINGMAN2/PhotoEncoder
cd PhotoEncoder
```

---
The Dockerfile is located in the `backend` directory. To build and run the backend with Docker:

```bash
cd backend
docker build -t backend .
docker run -p 8080:8080 backend
```

#### This starts the backend server at:
#### 👉 `http://localhost:8080`

---
## Using locally
The easiest way to use the backend is to `curl` it in the terminal

#### Using `curl`
```bash
// To encode
curl -X POST http://localhost:8080/encode   -F "image=@image.png"   -F "secret=@secret.txt"   -F "password=your_password_here"   --output encoded.png

// To decode. --output flag is optional
curl -X POST http://localhost:8080/decode   -F "image=@encoded.png"   -F "password=your_password_here2" --output output.txt
```

---

## ⚠️ Limitations

- Only supports images with sufficient capacity:  
  You can store approximately `width × height × 4 ÷ 8` bytes.
- No compression of the embedded message — you should compress it yourself if needed.
- Only works with PNGs or other formats that preserve pixel data exactly.
- Does not support embedding multiple secrets per image.

---

## 🔧 Dependencies

- [`image`](https://docs.rs/image) – for decoding and re-encoding PNG images.
- [`aes-gcm`](https://docs.rs/aes-gcm) – for AES-256-GCM encryption and decryption.
- [`sha2`](https://docs.rs/sha2) – for SHA-256 password hashing.
- [`rand`](https://docs.rs/rand) – for generating secure nonces.

---

## 📄 License

This project is licensed under the MIT License.

---

## ✍️ Author

Hashim Almuqbel
