use crate::err::{NfResult, NfError};
use crypto::{ symmetriccipher, buffer, aes, blockmodes, rc4 };
use crypto::symmetriccipher::SynchronousStreamCipher;
use crypto::buffer::{ ReadBuffer, WriteBuffer, BufferResult };


// static KEY: &[u8] = "1234567890qweewq32rtyuio432Tadfg".as_bytes();


// 加密方法
/// data: 明文
/// key: 长度为32
///
pub fn aes_encrypt(data: &[u8], key: String, iv: String) -> NfResult<Vec<u8>> {
    let mut encryptor = aes::cbc_encryptor(
        aes::KeySize::KeySize256,
        key.as_bytes(),
        iv.as_bytes(),
        blockmodes::PkcsPadding);

    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = buffer::RefReadBuffer::new(data);
    let mut buffer = [0; 4096];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = encryptor.encrypt(&mut read_buffer, &mut write_buffer, true)
            .map_err(|e| NfError::E(format!("aes encrypt error.")))?;

        // "write_buffer.take_read_buffer().take_remaining()" means:
        // from the writable buffer, create a new readable buffer which
        // contains all data that has been written, and then access all
        // of that data as a slice.
        final_result.extend(write_buffer.take_read_buffer().take_remaining().iter().map(|&i| i));

        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => { }
        }
    }

    Ok(final_result)
}

// 解密方法
/// data: 密文
/// iv:
pub fn aes_decrypt(data: &[u8], key: String, iv: String) -> NfResult<Vec<u8>> {
    let mut decryptor = aes::cbc_decryptor(
        aes::KeySize::KeySize256,
        key.as_bytes(),
        iv.as_bytes(),
        blockmodes::PkcsPadding);

    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = buffer::RefReadBuffer::new(data);
    let mut buffer = [0; 4096];
    let mut write_buffer = buffer::RefWriteBuffer::new(&mut buffer);

    loop {
        let result = decryptor.decrypt(&mut read_buffer, &mut write_buffer, true)
        .map_err(|e| NfError::E(format!("aes decrypt error.")))?;
        final_result.extend(write_buffer.take_read_buffer().take_remaining().iter().map(|&i| i));
        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => { }
        }
    }

    Ok(final_result)
}

// rc4流加密
pub fn rc4_encrypt(data: &[u8], key: String) -> NfResult<Vec<u8>> {
    // let mut r = rc4::Rc4::new("key".as_bytes());
    let mut r = rc4::Rc4::new(key.as_bytes());
    let length = data.len();
    let mut output = vec![0u8; length];
    // input 需要和output长度相等。
    r.process(data, output.as_mut_slice());
    // println!("output: {:?}", &output);
    Ok(output)
}

// rc4流解密
pub fn rc4_decrypt(data: &[u8], key: String) -> NfResult<Vec<u8>> {
    // let mut r = rc4::Rc4::new("key".as_bytes());
    let mut r = rc4::Rc4::new(key.as_bytes());
    let length = data.len();
    let mut output = vec![0u8; length];
    // input 需要和output长度相等。
    r.process(data, output.as_mut_slice());
    // println!("output: {:?}", &output);
    Ok(output)
}

