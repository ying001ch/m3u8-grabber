use anyhow::{Result, bail, anyhow};
use bytes::Bytes;
use reqwest::blocking::{Client, Response};
use std::{env, fmt::Debug, io::{Read, Write}, sync::{Arc, Mutex, RwLock}, time::Duration};
use crate::{async_runtime::{self, block_on}, config};

/// 静态变量
static ASYNC_CLIENT: RwLock<Option<reqwest::Client>> = RwLock::new(None);

/// 方法
#[test]
pub fn main() {
    let client = get_client();
    println!("{:?}", client);
}
pub async fn query_bytes_async<K,V>(url: &str, header: Option<&[(K, V)]>) ->std::result::Result<Bytes, String> 
where 
    K: AsRef<str> + Debug,
    V: AsRef<str> + Debug
    {
    let client = get_client();
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
                let resp = format!("{:?}", res);
                return Err(format!("=====> 请求异常,  resp: {:?} body:{:?},",resp, res.text().await));
            }
            res.bytes().await
                .map_err(|e| e.to_string())
        },
        Err(err) => {
            log::error!("error in query bytes: {:?}", err);
            Err(err.to_string())
        }
    }
}
pub fn query_bytes(url: &str, header: Option<&[(String, String)]>) ->anyhow::Result<Bytes> {
    let f = query_bytes_async(url, header);

    async_runtime::block_on(f).map_err(|e|anyhow!("{}",e))
}
pub fn query_text(url: &str, header: Option<&[(String, String)]>) -> Result<String> {
    let b = query_bytes(url, header);
    match b {
        Ok(res) => Ok(String::from_utf8_lossy(&res).to_string()),
        Err(err) => {
            log::error!("{}", err);
            // bail!();
            bail!("query text failed! err: {}",err)
        }
    }
}
fn get_client()-> reqwest::Client{
    let mut guard = ASYNC_CLIENT.read().unwrap();
    if guard.is_none() {
        drop(guard);
        update_client();
        guard = ASYNC_CLIENT.read().unwrap();
    }
    guard.as_ref().map(|f|f.clone()).unwrap()
}
pub fn update_client(){
    let mut guard = ASYNC_CLIENT.write().unwrap();
    *guard = Some(create_client());
    log::info!("update client success.")
}

fn create_client() -> reqwest::Client {
    let mut builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .danger_accept_invalid_certs(true) // 忽略证书验证
            .user_agent("MU1024/1.0")
            ;
    
    let p = get_proxy();
    if p.len()>0 {
        let proxy = reqwest::Proxy::all(p.as_str())
                .expect("socks proxy should be there");
        builder = builder.proxy(proxy);
        log::info!("use proxy: {}",p);
    }
    let cli = builder.build().expect("build clent failed.");
    cli
    }
fn get_proxy()-> String {
    config::get_proxys()
}