use crate::utils::crypt::{rc4_decrypt, rc4_encrypt, aes_encrypt, aes_decrypt};
use crypto::buffer;

#[test]
pub async fn test_block_encrypt() {
    let message = "Hello World!";

    // let mut key: [u8; 32] = [0; 32];
    let mut key = "1234567890qwertyuiopasdfghjklzxc".to_string();
    let mut iv = "qwertyuiopASDFGH".to_string();
    // let mut iv: [u8; 16] = [0; 16];

    // In a real program, the key and iv may be determined
    // using some other mechanism. If a password is to be used
    // as a key, an algorithm like PBKDF2, Bcrypt, or Scrypt (all
    // supported by Rust-Crypto!) would be a good choice to derive
    // a password. For the purposes of this example, the key and
    // iv are just random values.
    // let mut rng = OsRng::new().ok().unwrap();
    // rng.fill_bytes(&mut key);
    // rng.fill_bytes(&mut iv);

    let encrypted_data = aes_encrypt(message.as_bytes(), key.clone(), iv.clone()).unwrap();
    let origin_data = aes_decrypt(&encrypted_data[..], key.clone(), iv.clone()).unwrap();
    // let mut read_buffer = buffer::RefReadBuffer::new(data);
    // let mut buffer = [0; 4096];
    // let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);
    assert_eq!(message.as_bytes(), &origin_data[..]);
}


#[test]
pub async fn test_stream() {

    let key = "key".to_string();
    let input = "luna";
    let output = rc4_encrypt(input.as_bytes(), key.clone()).unwrap();
    println!("output: {:?}", &output);
    let origin = rc4_decrypt(&output[..], key.clone()).unwrap();
    println!("input: {:?}", &origin);
 
    assert_eq!(input.as_bytes(), origin);

}
