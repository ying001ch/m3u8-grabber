use anyhow::{Result, bail, anyhow};
use bytes::Bytes;
use reqwest::blocking::{Client, Response};
use std::{env, io::{Read, Write}, time::Duration, sync::{Mutex, Arc}};
use crate::{str_util, config};

/// 静态变量
static ASYNC_CLIENT: Mutex<Option<Arc<reqwest::Client>>> = Mutex::new(None);

/// 方法

pub fn main() {
    let args:Vec<String> = env::args().collect();

    query_bytes("http://localhost:8080/hs",0);
    println!("end..");
}
pub async fn query_bytes_async(url: &str, idx:i32) ->std::result::Result<Bytes, String> {
    let client = get_client2(0);
    let mut req_builder = client.get(url);
    let head = get_headers();
    for h in head {
        req_builder = req_builder.header(&h.0, &h.1);
    }
    let body = client.execute(req_builder.build().unwrap()).await;
    match body {
        Ok(res) => {
            if !res.status().is_success() {
                return Err(format!("=====> 请求异常，status: {}", res.status()));
            }
            res.bytes().await
                .map_err(|e|e.to_string())
        },
        Err(err) => {
            Err(err.to_string())
        }
    }
}
pub fn query_bytes(url: &str, idx:i32) ->std::result::Result<Box<Bytes>, reqwest::Error> {
    let client = get_client(idx);
    let mut req_builder = client.get(url);
    let head = get_headers();
    for h in head {
        req_builder = req_builder.header(&h.0, &h.1);
    }
    let body = client.execute(req_builder.build().unwrap());
    match body {
        Ok(res) => res.bytes().map(|b|Box::new(b)),
        Err(err) => {
            Err(err)
        }
    }
}
pub fn query_text(url: &str) -> Result<String> {
    let b = query_bytes(url,0);
    match b {
        Ok(res) => Ok(String::from_utf8_lossy(&res).to_string()),
        Err(err) => {
            println!("{}", err);
            // bail!();
            bail!("query text failed! err: {}",err)
        }
    }
}
fn get_client2(idx: i32)-> Arc<reqwest::Client>{
    let mut guard = ASYNC_CLIENT.lock().unwrap();
    if guard.is_none() {
        let mut builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(60));

        // let mut builder = reqwest::blocking::Client::builder();
        let p = get_proxy();
        if p.len()>0 {
            let proxy = reqwest::Proxy::all(p.as_str())
                    .expect("socks proxy should be there");
            builder = builder.proxy(proxy);
        }
        let cli = builder.build().expect("build clent failed.");
        *guard = Some(Arc::new(cli));
    }
    guard.as_ref().map(|f|f.clone()).unwrap()
    
}
pub fn update_client(){
    let mut guard = ASYNC_CLIENT.lock().unwrap();
    let mut builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(60));

    // let mut builder = reqwest::blocking::Client::builder();
    let p = get_proxy();
    if p.len()>0 {
        let proxy = reqwest::Proxy::all(p.as_str())
                .expect("socks proxy should be there");
        builder = builder.proxy(proxy);
    }
    let cli = builder.build().expect("build clent failed.");
    *guard = Some(Arc::new(cli));
    println!("=========> 更新proxy成功： proxy:{}",p);
}
fn get_client(idx: i32)-> Client{

    let mut builder = reqwest::blocking::Client::builder();
    let p = get_proxy();
    if p.len()>0 {
        let proxy = reqwest::Proxy::all(p.as_str())
                .expect("socks proxy should be there");
        builder = builder.proxy(proxy);
    }
    let cli = builder.build().expect("build clent failed.");
    cli
}

fn write_file(mut reader: Response) {
    let mut buf = [0u8; 1024 * 500];

    let mut file = std::fs::File::create("v.f56150——1.ts").expect("open file failed");
    loop {
        let res = reader.read(&mut buf);
        if let Ok(size) = res {
            println!("size is {}", size);
            if size <= 0 {
                break;
            }
            let handler = file.write(&buf[0..size]);
            handler.expect("写入失败");
            file.flush().expect("flush 失败");
        } else {
            panic!("读取失败");
        }
    }
}
fn get_proxy()-> String {
    config::get_proxys()
}
fn get_headers() -> Vec<(String, String)>{
    config::get_headers()
}