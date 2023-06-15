use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use super::http_util;

pub struct M3u8Entity{
    // content: String,
    pub method: String,
    pub key_url: String,
    pub key: [u8;16],
    pub iv: [u8;16],
    
    pub clip_urls: Vec<String>,
    pub url_prefix: Option<String>,
    pub save_path: Option<String>,
    pub temp_path: String
}
impl M3u8Entity {
    pub fn from(content: String) -> M3u8Entity {
        // let mut clip_urls = vec![];
        let method="".to_string();
        let key_url="".to_string();
        let key=[0;16];
        let iv=[0;16];
        let tt = timestamp1().to_string();
        let mut entity = M3u8Entity{
            clip_urls: vec![],
            url_prefix: None,
            method,
            key_url,
            key,
            iv,
            save_path:None,
            temp_path: tt
        };
        let lines  = content.lines();
        for li in lines {
            if li.contains("EXT-X-KEY"){
                // key method iv
                parse_Key(&mut entity, li);
            }else if li.contains(".ts") {
                entity.clip_urls.push(li.to_string());
            }
        }
        if entity.clip_urls.len()==0{
            panic!("M3U8 元信息解析错误，未解析到视频片段信息。content: \n{}", &content[0..200]);
        }
        if entity.key_url.len()==0 {
            println!("未发现密钥信息, 将不进行解密！");
        }
        println!("clip num: {}", entity.clip_urls.len());
        // temp_path
        let tpop = get_temp_path();
        if let Some(t) = tpop{
            entity.temp_path = t;
        }

        if !dir_exists(&entity.temp_path) {
            std::fs::create_dir(&entity.temp_path)
                    .expect("creat temp_path failed.");
        }
        println!("temp_path : {}", &entity.temp_path);

        
        entity
    }
    pub fn req_key(&mut self) {
        if !self.need_decode(){
            return;
        }

        let pr = env::args()
            .filter(|e| e.contains("--key"))
            .map(|e| e.replace("--key=", ""))
            .find(|e| true);
        if pr.is_some() {
            let bar = pr.unwrap().into_bytes();
            let mut k = [0u8;16];
            for i in 0..k.len(){
                k[i] = bar[i];
            }

            self.key = k;
            println!("key_bytes={:?}", self.key);
            return;
        }

        if !(&self.key_url).starts_with("http") {
            self.key_url = self.url_prefix.as_ref().unwrap().to_string() + &self.key_url;
        }
        println!("req_key key_url={}", &self.key_url);
        let raw_bytes = http_util::query_bytes(&self.key_url,0).unwrap();
        let mut key_bytes = [0u8;16];
        let len = raw_bytes.len();
        if len != 16 {
            panic!("reqKey failed");
        }
        let mut idx=0;
        for b in *raw_bytes {
            key_bytes[idx] = b;
            idx += 1;
        }
        self.key = key_bytes;
        println!("key_bytes={:?}", key_bytes);
    }
    fn to_string(&self)->String{
        format!("{{method={},key_url={},\nkey={:?},\niv={:?},\nclip_urls={:?}}}",
            self.method, self.key_url, self.key,self.iv, self.clip_urls)
    }
    pub fn need_decode(&self)-> bool{
        !self.key_url.is_empty()
    }
}
fn get_temp_path()-> Option<String>{
    std::env::args().filter(|e|e.contains("--temp="))
        .map(|e|e.replace("--temp=",""))
        .find(|e|true)
}
fn dir_exists(dir_path: &str)-> bool{
    let dir_ex = std::fs::read_dir(dir_path);
    println!("file_exists f={}, res: {}", dir_path, dir_ex.is_ok());
    dir_ex.is_ok()
}
fn timestamp1() -> i64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let ms = since_the_epoch.as_secs() as i64 * 1000i64 + (since_the_epoch.subsec_nanos() as f64 / 1_000_000.0) as i64;
    ms
}
fn parse_Key(mm: &mut M3u8Entity, line: &str) {
    let (k, vv) = line.split_once(":").unwrap();
    let keyStr = vv;
    let entrys = keyStr.split(",");
    for entry in entrys {
        let (x,y) = entry.split_once("=").unwrap();
        let val = y;
        if entry.starts_with("METHOD") {
            mm.method = val.to_string();
        }else if entry.starts_with("URI") {
            mm.key_url = val[1..val.len()-1].to_string();
        }else if entry.starts_with("IV") {
            mm.iv = hex2Byte(val);
        }
    }
}

fn hex2Byte(mut val: & str) -> [u8; 16] {
    if val.starts_with("0x") {
        val = &val[2..];
    }
    let nval = val.to_lowercase();
    // println!("{}", a);

    let length = val.len();
    let mut idx = 0;
    let mut bytes = [0u8; 16];
    while idx+2 <= length {
        let integer = from_hex(&nval[idx..idx+2]);
        bytes[idx/2] = integer;
        // bytes[idx/2] = integer/10*16 + integer % 10;
        idx += 2;
    }

    return bytes;
}

fn from_hex(idx: &str) -> u8 {
    let ac = idx.chars().next().unwrap();
    let ac2 = idx.chars().last().unwrap();

    let num:u8 = parse_hex_char(ac);
    let num2:u8 = parse_hex_char(ac2);

    num*16 + num2
}

fn parse_hex_char(ac: char) -> u8 {
    let mut ac = ac;
    if ac >= 'A' && ac <= 'C'{
        ac = (ac as u8 - 'A' as u8 + 'a' as u8) as char;
    }
    if !(ac >= 'a' && ac <= 'f') && !(ac >= '0' && ac <= '9'){
        panic!("解析数字错误:{}", ac);
    }
    let nu = ac as u8;
    if ac >= '0' && ac <= '9'{
        nu - ('0' as u8)
    }else {
        nu - ('a' as u8) + 10
    }
}