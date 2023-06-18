use std::collections::hash_map::DefaultHasher;
use std::env;
use std::fmt::format;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

use super::http_util;
use crate::str_util;
use crate::config;
use serde::{Serialize, Deserialize};

//下载任务 参数
#[derive(Serialize, Deserialize, Debug,Default)]
pub struct DownParam {
    pub address: String,
    pub save_path: String,
    pub proxy: Option<String>,        //http代理地址
    pub headers: Option<String>,      //请求头
    pub combine_dir: Option<String>,  //合并视片段的路径
    pub m3u8_file: Option<String>,    //m3u8文件路径
    pub temp_path: Option<String>,    //片段的临时存放目录
    pub key_str: Option<String>,      //m3u8片段的解密key
    pub worker_num: usize,            //下载任务使用的线程数量
    pub task_type: usize,            //任务类型，1-下载视频  2-合并现有目录下的视频片段
    pub no_combine: bool,            //任务类型，1-下载视频  2-合并现有目录下的视频片段
}
impl DownParam {
    pub fn from_cmd() -> Self{
        let mut param = DownParam::default();
        println!("===>param::default : {:?}", param);
        //获取命令
        let args: Vec<String> = env::args().collect();
        //下载地址
        param.address = args[1].clone();
        //任务类型
        param.task_type = config::TASK_DOWN;
        args.iter().for_each(|s|{
            if s.contains("--output="){ //保存路径
                param.save_path = s.replace("--output=","");
            }else if s.contains("--proxy="){ //http代理
                param.proxy = Some(s.replace("--proxy=",""));
            }else if s.contains("--H=") { //请求头
                param.headers = Some(s.replace("--H=", ""));
            }else if s.contains("--temp="){ //碎片文件存放目录
                param.temp_path = Some(s.replace("--temp=",""));
            }else if s.contains("--key="){ //解密Key
                param.key_str = Some(s.replace("--key=",""));
            }else if s.contains("--worker="){ //下载线程数
                param.worker_num = s.replace("--worker=", "")
                    .parse().unwrap_or(4);
            }else if s.contains("--noCombine"){ //只下载不合并
                param.no_combine = true;
            }else if s.contains("--file="){
                param.m3u8_file = Some(s.replace("--file=", ""));
            }else if s.contains("--combine="){
                param.combine_dir = Some(s.replace("--combine=", ""));
                param.task_type = config::TASK_COM;
            }
        });
        if param.worker_num <= 0 {
            param.worker_num = 4;
        }
        println!("===>param : {:?}", param);
        param
    }
}
//M3u8文件参数
#[derive(Debug)]
pub struct M3u8Entity{
    // content: String,
    pub method: String,
    pub key_url: String,
    pub key: [u8;16],
    pub iv: [u8;16],
    
    pub clip_urls: Vec<String>,
    pub url_prefix: Option<String>,
    pub save_path: String,
    pub temp_path: String
}
impl M3u8Entity {
    pub fn default() -> Self{
        let method="".to_string();
        let key_url="".to_string();
        let key=[0;16];
        let iv=[0;16];
        let tt = "".to_string();
        return M3u8Entity{
            clip_urls: vec![],
            url_prefix: None,
            method,
            key_url,
            key,
            iv,
            save_path: "".to_string(),
            temp_path: tt
        };
    }
    pub fn from(param: &DownParam) -> M3u8Entity {
        let content;
        let m3u8_file = param.m3u8_file.as_ref();
        if m3u8_file.is_some() && !m3u8_file.unwrap().is_empty(){
            //1. 解析m3u8文件
            content = std::fs::read_to_string(m3u8_file.unwrap()).unwrap();
        } else {
            //1. 解析m3u8文件
            let m3u8_url = param.address.as_str();
            content = http_util::query_text(m3u8_url);
        }


        // let mut clip_urls = vec![];
        let mut entity = Self::default();
        entity.temp_path = cal_hash(&param.address);
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
        if let Some(t) = param.temp_path.as_ref().filter(|f|!f.is_empty()){
            entity.temp_path = t.to_string();
        }

        if !dir_exists(&entity.temp_path) {
            std::fs::create_dir(&entity.temp_path)
                    .expect("creat temp_path failed.");
        }
        println!("temp_path : {}", &entity.temp_path);

        entity.save_path = param.save_path.to_string();

        //----------------------------------------------------------------
        entity.process(param);
        entity
    }
    /**
     * 处理urlPrefix 和 获取解密key
     */
    fn process(&mut self, param :&DownParam) {
        let m3u8_url = param.address.as_str();
        let mut idx1: i32 = str_util::index_of('?', m3u8_url);
        if idx1 == -1 {
            idx1 = m3u8_url.len() as i32;
        }
        let idx2 = str_util::last_index('/', &m3u8_url[0..idx1 as usize]);
        if idx2 == -1 {
            panic!("最后一个 / 找不到");
        }
        self.url_prefix = Some((&m3u8_url[0..idx2 as usize]).to_string() + "/");
        println!("url_prefix = {}", self.url_prefix.as_ref().unwrap());
    
        self.req_key(param);
    }
    pub fn req_key(&mut self, param :&DownParam) {
        if !self.need_decode(){
            return;
        }

        let pr = param.key_str.as_ref();
        if pr.filter(|f|!f.is_empty()).is_some() {
            let bar = pr.unwrap().to_string().into_bytes();
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
fn cal_hash(input : &str) -> String{
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let output = hasher.finish();
    println!("auto temp clip dir ={}", output); // 输出字符串的哈希值
    format!("{}",output)
}