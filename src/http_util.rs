use reqwest::{blocking::{Client, Response}};
use std::{env, io::{Read, Write}};
use bytes::Bytes;
use crate::str_util;
use std::thread::Thread;
use std::collections::HashMap;


pub struct ReqParam{
    proxys: String,
    headers: Vec<(String,String)>,
}
static mut req_param :ReqParam = 
    ReqParam{proxys: String::new(),
     headers: vec![],
    };
pub fn main() {
    let args:Vec<String> = env::args().collect();
    set_header(&args);

    query_bytes("http://localhost:8080/hs",0);
    println!("end..");
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
pub fn query_text(url: &str) ->String {
    let b = query_bytes(url,0);
    match b {
        Ok(res) => String::from_utf8_lossy(&res).to_string(),
        Err(err) => {
            println!("{}", err);
            panic!("query text failed!");
        }
    }
}
fn get_client(idx: i32)-> Client{
    // static mut map:Option<HashMap<i32, Client>> = None;
    // let mut m;
    // unsafe{
    //     if let None = map{
    //         map = Some(HashMap::new());
    //     }
    //     m = map.as_mut().unwrap();
    //     // if m.contains_key(&idx) {
    //     //     return m.get(&idx).as_ref().unwrap();
    //     // }
    // }

    let mut builder = reqwest::blocking::Client::builder();
    if get_proxy().len()>0 {
        let proxy = reqwest::Proxy::all(get_proxy())
                .expect("socks proxy should be there");
        builder = builder.proxy(proxy);
    }
    let cli = builder.build().expect("build clent failed.");
    cli
    // unsafe {
    //     m.insert(idx, cli);
    //     println!("new http client for thread: {}", idx);
    //     m.get(&idx).as_ref().unwrap()
    // }
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
fn get_proxy()->&'static str{
    unsafe {
        &req_param.proxys
    }
}
pub fn set_proxy(proxy_s: String){
    println!("proxy={}", &proxy_s);
    unsafe {
        // proxys = proxy_s;
        req_param.proxys = proxy_s;
    }
}
pub fn set_header(args: &[String]){
    let headers:Vec<(String,String)> = args.iter()
            .filter(|&e|e.starts_with("--H=")&&e.contains(":"))
            .map(|e|e.replace("--H=",""))
            .map(|e|{
                let idx = str_util::index_of(':', &e) as usize;
                let k = &e[0..idx];
                let v = &e[idx+1..e.len()];
                (k.trim().to_string(),v.trim().to_string())
            }).collect();
    println!("headers: {:?}",headers);
    unsafe {
        req_param.headers=headers;        
    }
}
fn get_headers() -> &'static Vec<(String, String)>{
    unsafe {
        &req_param.headers      
    }
}