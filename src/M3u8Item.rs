use core::panic;
use std::collections::hash_map::DefaultHasher;
use std::path::Path;
use std::{default, env};
use std::error::Error;
use std::fmt::format;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

use super::http_util;
use crate::config;
use anyhow::{anyhow, bail, Context, Result};
use m3u8_rs::MediaPlaylist;
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
    pub worker_num: usize,            //下载使用的并行任务数量 async方式
    pub task_type: usize,            //任务类型，1-下载视频  2-合并现有目录下的视频片段
    pub combine_type: usize,            //合并类型 1-二进制合并 2-ffmpeg合并
    pub no_combine: bool,            //任务类型，1-下载视频  2-合并现有目录下的视频片段
}
impl DownParam {
    pub fn from_cmd() -> Self{
        let mut param = DownParam::default();
        //获取命令
        let args: Vec<String> = env::args().collect();
        //下载地址
        param.address = args[1].clone();
        //任务类型
        param.task_type = config::TASK_DOWN;
        param.combine_type = config::COMB_BIN;
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
                    .parse().unwrap_or(80);
            }else if s.contains("--noCombine"){ //只下载不合并
                param.no_combine = true;
            }else if s.contains("--file="){
                param.m3u8_file = Some(s.replace("--file=", ""));
            }else if s.contains("--combine="){
                param.combine_dir = Some(s.replace("--combine=", ""));
                param.task_type = config::TASK_COM;
            }else if s.contains("--combine_type="){
                match s.replace("--combine_type=", "").parse::<usize>() {
                    std::result::Result::Ok(num) => param.combine_type = num,
                    Err(_) => log::error!("无法将字符串转换为usize类型: {}", s),
                };
            }
        });
        if param.worker_num <= 0 {
            param.worker_num = 8;
        }
        log::info!("===>param : {:?}", param);
        param
    }
}
//M3u8文件参数
#[derive(Debug,Default)]
pub struct M3u8Entity{
    // content: String,
    pub media_play_list: MediaPlaylist,
    pub key: [u8;16],
    pub iv: [u8;16],
    pub key_num: usize,

    pub headers: Vec<(String,String)>,
    pub url_prefix: Option<String>,
    pub save_path: String,
    pub temp_path: String
}
impl M3u8Entity {
    pub fn from(param: &DownParam) -> Result<M3u8Entity> {
        let m3u8_file = param.m3u8_file.as_ref();
        let content = 
        if m3u8_file.is_some() && !m3u8_file.unwrap().is_empty(){
            std::fs::read_to_string(m3u8_file.unwrap())?
        } else {
            //1. 解析m3u8文件
            let m3u8_url = param.address.as_str();
            http_util::query_text(m3u8_url)?
        };


        // let mut clip_urls = vec![];
        let mut entity = Self::default();
        // temp_path
        entity.temp_path = param.temp_path.clone()
            .filter(|f|!f.is_empty())
            .unwrap_or_else(||cal_hash(&param.address));

        if !Path::new(&entity.temp_path).exists() {
            std::fs::create_dir_all(&entity.temp_path)
                .context(format!("create temp path failed. {}", &entity.temp_path))?;
        }
        log::info!("temp_path : {}", &entity.temp_path);
        // save_path
        entity.save_path = param.save_path.to_owned();

        // 使用m3u8-rs解析m3u8文件
    // 使用m3u8-rs解析m3u8文件
        let m3u8_result = m3u8_rs::parse_media_playlist_res(content.as_bytes());
        match m3u8_result {
            Ok(play_list) => {
                entity.media_play_list = play_list;
            },
            Err(e) => {
                bail!("M3U8 解析错误: {}", &e.to_string()[..100])
            },
        }

        let clips = &entity.media_play_list.segments;
        if clips.is_empty(){
            bail!(format!("M3U8 元信息解析错误，未解析到视频片段信息。content: \n{}", &content[0..200]));
        }
        if clips[0].key.as_ref().filter(|&k|k.uri.is_some()).is_none(){
            log::info!("未发现密钥信息, 将不进行解密！");
        }
        entity.key_num = clips.iter()
            .filter(|&e|
                e.key.as_ref()
                    .and_then(|f|f.uri.as_ref())
                    .is_some()
            )
            .count();
        log::info!("clip num: {}", clips.len());

        //----------------------------------------------------------------
        entity.process(param)?;

         //设置请求头
        param.headers.as_ref()
            .filter(|&f|!f.is_empty())
            .inspect(|&h|{
                let v = h.split(";;")
                    .map(|h|{
                        match h.find(':') {
                            Some(idx) => {
                                let k = &h[0..idx];
                                let v = &h[idx+1..h.len()];
                                (k.trim().to_string(),v.trim().to_string())
                            },
                            None => {
                                (h.trim().to_string(),String::new())
                            }
                        }
                    })
                    .collect();
                log::info!("headers is :{:?}", v);
                entity.headers = v;
            });
        
        Ok(entity)
    }
    /**
     * 处理urlPrefix 和 获取解密key
     */
    fn process(&mut self, param :&DownParam) -> Result<()> {
        let m3u8_url = param.address.as_str();
        //找到?位置 如果找不到就返回长度
        let idx1 = m3u8_url.find('?').unwrap_or(m3u8_url.len());
        //找到path部分最后一个 /
        let idx2 = (&m3u8_url[0..idx1]).rfind('/').ok_or(anyhow!("M3u8地址最后一个 / 找不到"))?;

        self.url_prefix = Some((&m3u8_url[0..idx2]).to_string() + "/");
        log::info!("url_prefix = {}", self.url_prefix.as_ref().unwrap());

        self.req_key(param)
    }
    pub fn req_key(&mut self, param :&DownParam) -> Result<()>{
        if !self.need_decode(){
            return Ok(());
        }

        if let Some(ref s) = param.key_str {
            self.key = hex2_byte(s).map_err(|e: anyhow::Error|{
                anyhow!(format!("key_str 解析错误: {}", e))
            })?;
            return Ok(());
        }

        let first_key = self.media_play_list.segments[0].key.as_ref().unwrap();
        let mut key_url = first_key.uri.clone().unwrap();
        if !&key_url.starts_with("http") {
            key_url = self.url_prefix.as_ref().unwrap().to_string() + &key_url;
        }
        log::info!("req_key key_url={}", key_url);
        let raw_bytes = http_util::query_bytes(&key_url)?;
        if raw_bytes.len() != 16 {
            bail!("requested key length is not 16")
        }
        self.key.copy_from_slice(&raw_bytes);

        self.iv = hex2_byte(first_key.iv.as_ref().unwrap())?;
        log::info!("key_bytes={:?}", self.key);
        Ok(())
    }
    pub fn need_decode(&self)-> bool{
        self.key_num > 0
    }
    pub fn multi_key(&self)-> bool{
        self.key_num > 1
    }
    pub fn clip_num(&self) -> usize{
        self.media_play_list.segments.len()
    }
}
fn parse_key(mm: &mut M3u8Entity, line: &str) {
    let (_k, vv) = line.split_once(":").unwrap();
    let key_str = vv;
    let entrys = key_str.split(",");
    for entry in entrys {
        let (_x,y) = entry.split_once("=").unwrap();
        let val = y;
        if entry.starts_with("METHOD") {
            // mm.method = val.to_string();
        }else if entry.starts_with("URI") {
            // mm.key_url = val[1..val.len()-1].to_string();
        }else if entry.starts_with("IV") {
            mm.iv = hex2_byte(val).unwrap();
        }
    }
}

pub fn hex2_byte(mut val: & str) -> Result<[u8; 16]> {
    if val.starts_with("0x") {
        val = &val[2..];
    }
    if val.len() != 32{
        bail!("hex2_byte: len != 16 val: {}", val);
    }
    let nval = val.to_lowercase();

    let length = val.len();
    let mut idx = 0;
    let mut bytes = [0u8; 16];
    while idx+2 <= length {
        bytes[idx/2] = u8::from_str_radix(&nval[idx..idx+2], 16)?;
        idx += 2;
    }

    return Ok(bytes);
}
fn cal_hash(input : &str) -> String{
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let output = hasher.finish();
    log::info!("auto temp clip dir ={}", output); // 输出字符串的哈希值
    format!("{}",output)
}