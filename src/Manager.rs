use anyhow::anyhow;
use bytes::Bytes;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::runtime::Builder;
use tokio::sync::Semaphore;
use tokio::task::AbortHandle;
use tokio::task::JoinHandle;
use anyhow::Result;

use crate::aes_util;
use crate::combine;
use crate::config::Signal;
use crate::http_util;
use crate::M3u8Item;
use crate::M3u8Item::DownParam;
use crate::str_util;
use crate::config;
use std::fmt::format;
use std::io::Error;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;

pub fn run(param: DownParam, async_task: bool) -> Result<()>{
    println!("Hello this is M3u8-Downloader by rust");

    //设置代理和请求头
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
    
    
    let entity = M3u8Item::M3u8Entity::from(&param)?;
    config::add_task(&entity.temp_path); //使用片段临时路径 创建任务状态信息
    let one = move ||{
        let temp_path = entity.temp_path.clone();
        let save_path = entity.save_path.clone();
        let st = SystemTime::now(); //计时开始
        //手动创建了运行时，就可以不再使用main方法上的注解
        tokio::runtime::Runtime::new().unwrap()
                .block_on(download_async(entity));
        let spend_time = st.elapsed().unwrap().as_secs();

        if config::is_abort(&temp_path){
            println!("--->下载暂停");
            return ;
        }
        println!("下载完毕！总耗时：{}s no_combine:{}", spend_time, param.no_combine);

        //合并片段
        if !param.no_combine {
            combine::combine_clip(temp_path.as_str(), save_path.as_str());
        }
    };
    let handle = thread::spawn(one);
    if !async_task {
        handle.join().map_err(|e|anyhow!(format!("{:?}",e)))?;
    }
    Ok(())
}
async fn download_async(entity: M3u8Item::M3u8Entity){
    //设置进度
    config::init_progress(entity.clip_urls.len());

    let clip_urls =  entity.clip_urls.clone();
    let temp_path = entity.temp_path.clone();
    let nd = entity.need_decode();
    let key = entity.key;
    let iv = entity.iv;
    let prefix = entity.url_prefix.as_ref().unwrap().clone();
    let mut join_v = vec![];
    let semaphore = Arc::new(Semaphore::new(config::get_work_num()));
    let err_vec: Vec<usize> = vec![];
    let err_clips = Arc::new(Mutex::new(err_vec));
    for idx in 0..clip_urls.len() {
        let clip_clone = clip_urls[idx].clone();
        // let clip_clone = clip.clone();
        let prefix_clone = prefix.to_string();
        let temp_path_clone = temp_path.clone();
        let sem = semaphore.clone();
        let err_clone = Arc::clone(&err_clips);
        let handler = tokio::spawn(async move{
            if config::is_abort(&temp_path_clone){
                return;
            }
            let permit = sem.acquire().await.unwrap();
            //判断是否停止
            if config::is_abort(&temp_path_clone){
                return;
            }
            let down_file_path = format!("{}/{}.ts", temp_path_clone, make_name(idx as i32 +1));
            if tokio::fs::File::open(down_file_path.clone()).await.is_ok() {
                //文件已经存在，无需下载
                config::add_prog();
                drop(permit);
                return;
            }

            let down_url = prefix_clone.to_string() + clip_clone.as_str();
            // println!("--> {}", down_url);

            let mut bytes = http_util::query_bytes_async(&down_url,0 as i32).await;
            if config::is_abort(&temp_path_clone){
                println!("收到停止信号，不再下载");
                return;
            }
            let mut err_num = 1;
            while let Err(err) = bytes {
                println!("下载片段({})出错：{}, err_num={}", idx, err, err_num);
                if err_num >=5 {
                    err_clone.lock().unwrap().push(idx);
                    drop(permit);
                    return;
                }
                // put_retry(&mut retry_num, &clone_pkg, clip_index, &clip);
                bytes = http_util::query_bytes_async(&down_url, 0 as i32).await;
                err_num += 1;
            }
            println!("片段({})下载完成 len: {}", idx, bytes.as_ref().map(|op|op.len()).unwrap());
            //写入文件
            let temp;
            let result: &[u8] = if nd {
                let res = aes_util::decrypt(bytes.as_ref().unwrap(), &key, &iv);
                if let Ok(v) = res{
                    temp = v;
                    &temp
                }else{
                    println!("片段({}) Decode ERROR 解密过程出错：{}", idx, res.unwrap_err());
                    drop(permit);
                    return;
                }
            } else {
                bytes.as_ref().unwrap()
            };
            let res = write_file_async(result, down_file_path)
                    .await;
            if let Err(e) = res{
                println!("写入片段[{}]失败， err={}", idx + 1, e);
            }else{
                config::add_prog();
                println!("写入片段[{}]成功！", idx + 1);
            }
            drop(permit);
        });
        join_v.push(handler);
        //限制并发量的一种方法，分组执行，有一定效果
        //但是必须等到一组都执行完毕才能往下走，效率有些波动
        //后续可以再优化一下
        // if join_v.len() >= 1000 {
        //     let join_v_2 = join_v;
        //     join_v = vec![];
        //     exec_group(join_v_2).await;
        // }
    }
    println!("join_v len = {}", join_v.len());
    // 创建监控线程，abort()任务
    let abort_v:Vec<AbortHandle> = join_v.iter()
            .map(|j|j.abort_handle())
            .collect();
    let temp_clone = temp_path.clone();
    tokio::spawn(async move{
        while config::is_normal(&temp_clone) {
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
        println!("=====>停止任务执行, status: {:?}", config::get_status(&temp_clone));
        abort_v.iter().filter(|a|!a.is_finished())
            .for_each(|a|a.abort());
    });

    let mut idx = 1;
    for j in join_v{
        // println!("===> handler={} 开始执行",idx);
        j.await.unwrap();
        // println!("===> handler={} 执行结束", idx);
        idx += 1;
    }
    
    if err_clips.lock().unwrap().len() > 0{
        println!("以下片段出错没有下载完成: {:?}", err_clips.lock().unwrap());
    }
    //TODO 正常下载完成时设置标记为end
    if config::is_normal(&temp_path){
        config::set_signal(&entity.temp_path, Signal::End);
    }
}
async fn exec_group(join_v: Vec<JoinHandle<()>>){
    for j in join_v.into_iter() {
        println!("===> handler= 开始执行");
        j.await;
        println!("===> handler= 执行结束");
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

async fn write_file_async(result: &[u8], f_name: String) -> Result<(), Error> {
    let mut f = File::create(f_name.clone()).await.unwrap();

    let n = f.write(result).await.unwrap();
    if let Err(e) = f.flush().await{
        return Err(e);
    }
    println!("write {} bytes", n);

    println!("写入成功 counter:{}, size: {}", f_name, n);
    Ok(())
}
fn write_file(result: &[u8], entity: &M3u8Item::M3u8Entity, file_name: String) {
    let save_path = entity.temp_path.clone();

    let mut file =
        std::fs::File::create(format!("{}/{}.ts", save_path, file_name)).expect("open file failed");
    let usize = file.write(result).expect("写入文件失败");
    // println!("写入成功 counter:{}, size: {}", file_name, usize);
}
