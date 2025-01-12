use anyhow::{Result, bail, anyhow};
use bytes::Bytes;
use reqwest::blocking::{Client, Response};
use std::{env, fmt::Debug, io::{Read, Write}, sync::{Arc, Mutex}, time::Duration};
use crate::{async_runtime::{self, block_on}, config};

/// 静态变量
static ASYNC_CLIENT: Mutex<Option<reqwest::Client>> = Mutex::new(None);

/// 方法
#[test]
pub fn main() {
    let fc = ||{
        let text = query_text("https://baidu.com");
        if let Ok(res) = text {
            println!("res: {}", &res[..100]);
        } else {
            println!("error: {:?}", text);
        }
    };
    fc();
    fc();
    println!("end..");
}
pub async fn query_bytes_async<K,V>(url: &str, header: Option<&[(K, V)]>) ->std::result::Result<Bytes, String> 
where 
    K: AsRef<str> + Debug,
    V: AsRef<str> + Debug
    {
    let client = get_client2();
    let mut req_builder = client.get(url);
    if let Some(header) = header {
        log::debug!("自定义请求头：{:?}", header);
        for (k,v) in header {
            req_builder = req_builder.header(k.as_ref(), v.as_ref());
        };
    };
    
    let body = req_builder.send().await;
    match body {
        Ok(res) => {
            if !res.status().is_success() {
                return Err(format!("=====> 请求异常，status: {}", res.status()));
            }
            res.bytes().await
                .map_err(|e| e.to_string())
        },
        Err(err) => {
            Err(err.to_string())
        }
    }
}
pub fn query_bytes(url: &str) ->anyhow::Result<Bytes> {
    let f = query_bytes_async::<&str,&str>(url, None);

    async_runtime::block_on(f).map_err(|e|anyhow!("{}",e))
}
pub fn query_text(url: &str) -> Result<String> {
    let b = query_bytes(url);
    match b {
        Ok(res) => Ok(String::from_utf8_lossy(&res).to_string()),
        Err(err) => {
            log::error!("{}", err);
            // bail!();
            bail!("query text failed! err: {}",err)
        }
    }
}
fn get_client2()-> reqwest::Client{
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
        *guard = Some(cli);
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
    *guard = Some(cli);
}
fn get_proxy()-> String {
    config::get_proxys()
}