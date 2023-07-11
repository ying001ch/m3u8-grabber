use std::vec;

use aes::Aes128;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};

use crate::M3u8Item;

// create an alias for convenience
type Aes128Cbc = Cbc<Aes128, Pkcs7>;

fn encrypt(content: &[u8], key:&[u8], iv:&[u8]) -> Vec<u8>{
    let cipher = Aes128Cbc::new_from_slices(&key, &iv).unwrap();

    // buffer must have enough space for message+padding
    // copy message to the buffer
    let result = cipher.encrypt_vec(content);
    result
}
pub fn decrypt(encry_content: &[u8], key:&[u8], iv:&[u8]) -> Result<Vec<u8>, String>{
    let cipher = Aes128Cbc::new_from_slices(key, iv).unwrap();
    match cipher.decrypt_vec(encry_content){
        Ok(t)=>Ok(t),
        Err(e)=>Err(e.to_string())
    }
}
#[test]
fn test_decrypt(){
    use base64::{Engine as _, alphabet, engine::{self, general_purpose}};

    let content = std::fs::read(r"D:\Kitchen\Repository\tauri\crawler\response.ts").unwrap();
    let key = M3u8Item::hex2_byte("aebae151fbaed00fb50be634d850b7b0");
    //base64格式的密钥
    // let key =  general_purpose::STANDARD.decode("rrrhUfuu0A+1C+Y02FC3sA==").unwrap();
    let iv  = M3u8Item::hex2_byte("0x00000000000000000000000000000000");

    println!("key: {} iv: {}",key.len(),iv.len());
    println!("file len: {} mod(16):{}", content.len(), content.len()%16);
    
    decrypt(&content, &key, &iv).map(|res|{
        std::fs::write(r"C:\Users\frank\Desktop\response_dec.ts", &res).expect("写入文件失败");
        println!("解密写入成功");
    }).expect("解密失败");
}