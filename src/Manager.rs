use bytes::Bytes;

use crate::aes_demo;
use crate::combine;
use crate::http_util;
use crate::M3u8Item;
use crate::M3u8Item::DownParam;
use crate::str_util;
use crate::config;
use core::panic;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::env;
use std::io::Write;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

pub fn run(param: DownParam) {
    println!("Hello this is M3u8-Downloader by rust");

    param.proxy.as_ref()
        .filter(|f|!f.is_empty())
        .map(|p|config::set_proxys(p.to_string()));
    param.headers.as_ref()
        .filter(|&f|!f.is_empty())
        .map(|h|{
            let v = h.split(";")
                .map(|e|{
                    let idx = str_util::index_of(':', e) as usize;
                    let k = &e[0..idx];
                    let v = &e[idx+1..e.len()];
                    (k.trim().to_string(),v.trim().to_string())
                })
                .collect();
            config::set_headers(v);
        });
    //set workerNum
    config::set_work_num(param.worker_num);
    
    // http_util::set_header(&args);


    let entity = M3u8Item::M3u8Entity::from(&param);

    let temp_path = entity.temp_path.clone();
    let save_path = entity.save_path.clone();
    download_decode(entity);

    println!("下载完毕！no_combine:{}", param.no_combine);
    if !param.no_combine {
        combine::combine_clip(temp_path.as_str(), save_path.as_str());
    }
}

fn download_decode(entity: M3u8Item::M3u8Entity) {
    // println!("savePath={}", entity.save_path.as_ref().unwrap());
    let entity_it = Arc::new(entity);

    let mut pkg = vec![];
    let len = entity_it.clip_urls.len();
    for i in 0..len {
        pkg.push(((len - i) as i32, entity_it.clip_urls[len-1-i].clone(),0));
    }
    let pkg = Arc::new(Mutex::new(pkg));

    let (tx, rx) = mpsc::channel::<(Box<Bytes>,i32)>();
    
    let mut write_worker = vec![];
    // 创建解密和 写文件线程
    let rx_holder = Arc::new(Mutex::new(rx));
    for i in 0..2 {
        let holder = Arc::clone(&rx_holder);
        let clone_entity = Arc::clone(&entity_it);
        let handler = thread::spawn(move||{
            let dd = clone_entity.as_ref();
            let key = &dd.key;
            let iv = &dd.iv;
            
            loop{
                let (byte_vec, clip_index) =  {
                    holder.lock().unwrap().recv().unwrap()
                };
                if clip_index<0 {
                    println!("退出线程====");
                    break;
                }

                println!("vec size = {}", byte_vec.len());

                let temp;
                let result: &[u8] = if dd.need_decode() {
                    let res = aes_demo::decrypt(&byte_vec, key, iv);
                    if let Ok(v) = res{
                        temp = v;
                        &temp
                    }else{
                        println!("Decode ERROR 解密过程出错：{}", res.unwrap_err());
                        continue;
                    }
                } else {
                    byte_vec.as_ref()
                };

                write_file(result, &dd, make_name(clip_index));
                println!("{}--写入成功！clip_index={}\n",i, clip_index);
            }
        });
        write_worker.push(handler);
    }

    // 下载线程
    let mut download_worker = vec![];
    for i in 0..config::get_work_num() {
        let clone_entity = Arc::clone(&entity_it);
        let clone_pkg = Arc::clone(&pkg);
        let txc = tx.clone();
        let handler = thread::spawn(move || {
            sleep(Duration::from_millis(i as u64 *300u64));
            let dd = clone_entity.as_ref();
            let prefix = dd.url_prefix.as_ref().unwrap();
            loop {

                let clip;
                let clip_index;
                let mut retry_num ;
                {
                    let mut pkd_ref = clone_pkg.lock().unwrap();
                    let (clip_index_, clip_, retry_num_) = match pkd_ref.pop(){
                        Some(e)=>e,
                        None=>break
                    };
                    clip = clip_;
                    clip_index = clip_index_;
                    retry_num = retry_num_;
                    if retry_num > 0{
                        println!("错误片段重新下载。retry_num={}", retry_num);
                    }
                }
                if file_exists(format!(
                    "{}/{}.ts",
                    dd.temp_path,
                    make_name(clip_index)
                ).as_ref()) {
                    continue;
                }

                let down_url = prefix.to_string() + clip.as_str();
                println!("--> {}", down_url);

                let result = http_util::query_bytes(&down_url,i as i32);
                if result.is_err() {
                    put_retry(&mut retry_num, &clone_pkg, clip_index, &clip);
                    println!("下载出错：{}", result.unwrap_err());
                    continue;
                }
                txc.send((result.unwrap(), clip_index)).unwrap();
                
                println!("{}--下载成功！clip_index={}\n",i, clip_index)
            }
        });
        download_worker.push(handler);
    }
    for ha in download_worker {
        ha.join().expect("线程被中断");
    }

    for _ha in 0..write_worker.len() {
        tx.send((Box::new(Bytes::new()), -1)).unwrap();
    }
    for ha in write_worker {
        ha.join().expect("线程被中断");
    }
    
}

fn file_exists(file_path: &str)-> bool{
    let file_ex = std::fs::File::open(file_path);
    file_ex.is_ok()
}

fn put_retry(retry_num: &mut i32, clone_pkg: &Arc<Mutex<Vec<(i32, String, i32)>>>, 
        clip_index: i32, clip: &String) {
    if *retry_num < 3{
        let mut pkd_ref = clone_pkg.lock().unwrap();
        let len = pkd_ref.len();
        *retry_num += 1;
        pkd_ref.insert(len/2, (clip_index, clip.to_string(), *retry_num));
    }
}
fn get_thread_num()->u8{
    let num = std::env::args().filter(|e|e.contains("--worker="))
        .map(|e|e.replace("--worker=",""))
        .map(|e|->u8 {e.parse().expect("")})
        .find(|e|true)
        .unwrap_or(4);
    println!("worker num={}", num);
    num
}

fn make_name(num: i32) -> String {
    if num < 1000 {
        let s = format!("{}", num);
        let pad = "0".repeat(4 - s.len()) + &s;

        return pad;
    }
    format!("{}", num)
}

fn write_file(result: &[u8], entity: &M3u8Item::M3u8Entity, file_name: String) {
    let save_path = entity.temp_path.clone();

    let mut file =
        std::fs::File::create(format!("{}/{}.ts", save_path, file_name)).expect("open file failed");
    let usize = file.write(result).expect("写入文件失败");
    // println!("写入成功 counter:{}, size: {}", file_name, usize);
}
