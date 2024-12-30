use anyhow::anyhow;
use anyhow::bail;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;
use tokio::task::AbortHandle;
use tokio::task::JoinHandle;
use anyhow::Result;

use crate::aes_util;
use crate::async_runtime;
use crate::combine;
use crate::config::Signal;
use crate::http_util;
use crate::M3u8Item;
use crate::M3u8Item::DownParam;
use crate::config;
use std::io::Error;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;

use std::thread;
use std::time::SystemTime;

/// 决定任务是 异步还是同步，合并文件还是下载文件
pub fn dispatch(param: DownParam, async_task: bool) -> Result<()>{
    // 校验参数
    validate_param(&param)?;
    match param.task_type {
        //下载任务
        config::TASK_DOWN => run(param, async_task),
        //合并任务
        config::TASK_COM => combine::combine_clip(
            param.combine_dir.unwrap().as_str(),
            &param.save_path.as_str(),
            param.combine_type,
            async_task),
        _=> bail!("任务类型不对"),
    }
}
/// 校验参数
fn validate_param(param: &DownParam)-> Result<()>{
    //校验 合并参数、下载参数
    match param.task_type {
      config::TASK_COM => {
        param.combine_dir.as_ref().ok_or(anyhow!("合并目录未指定")).map(|_|())
      },
      config::TASK_DOWN=>{
        if param.address.is_empty() || !param.address.starts_with("http"){
          bail!("下载地址格式不正确")
        }
        Ok(())
      },
      _=> bail!("任务类型不对")
    }
}
/// 运行下载任务
fn run(param: DownParam, async_task: bool) -> Result<()>{
    println!("Hello this is M3u8-Downloader by rust");

    //设置代理 
    param.proxy.as_ref()
        .filter(|f|!f.is_empty())
        .map(|p|config::set_proxys(p.to_string()))
        .or_else(||{
            config::set_proxys("".to_string());
            None
        });
    //设置请求头
    param.headers.as_ref()
        .filter(|&f|!f.is_empty())
        .map(|h|{
            let v = h.split(";")
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
            println!("headers is :{:?}", v);
            config::set_headers(v);
        });
    //set workerNum
    config::set_work_num(param.worker_num);
    
    
    let entity = M3u8Item::M3u8Entity::from(&param)?;
    config::add_task(&entity)?; //使用片段临时路径 创建任务状态信息
    let one = async move {
        let entity = &entity;
        let temp_path = entity.temp_path.as_str();
        let save_path = entity.save_path.as_str();
        let st = SystemTime::now(); //计时开始
        //手动创建了运行时，就可以不再使用main方法上的注解
        let all_success = download_async(entity).await;
        let spend_time = st.elapsed().unwrap().as_secs();

        println!("status is {:?}", config::get_status(temp_path));
        if config::is_abort(temp_path){
            println!("--->下载暂停");
            return ;
        }
        println!("下载完毕！总耗时：{}s no_combine:{} all_success:{}", spend_time, param.no_combine, all_success);

        //合并片段
        if all_success && !param.no_combine {
            combine::combine_clip(temp_path, save_path,param.combine_type, false).unwrap();
        }
    };
    if async_task{
        async_runtime::spawn(one);
    }else{
        async_runtime::block_on(one);
    }
    Ok(())
}
///异步下载方法
async fn download_async(entity: &M3u8Item::M3u8Entity) -> bool {
    let clip_urls =  &entity.clip_urls;
    let temp_path = &entity.temp_path;
    let nd = entity.need_decode();
    let key = entity.key;
    let iv = entity.iv;

    let prefix = entity.url_prefix.as_ref().unwrap();
    let mut join_v = vec![];
    let semaphore = Arc::new(Semaphore::new(config::get_work_num()));
    let err_vec: Vec<usize> = vec![];
    let err_clips = Arc::new(Mutex::new(err_vec));
    for idx in 0..clip_urls.len() {
        let clip_clone = clip_urls[idx].clone();
        // let clip_clone = clip.clone();
        let prefix = prefix.to_string();
        let temp_path = temp_path.clone();
        let sem = semaphore.clone();
        let err_clips = Arc::clone(&err_clips);
        let handler = tokio::spawn(async move{
            let _permit = sem.acquire().await.unwrap();
            let down_file_path = format!("{}/{}.ts", temp_path, make_name(idx +1));
            if tokio::fs::File::open(down_file_path.as_str()).await.is_ok() {
                //文件已经存在，无需下载
                config::add_prog(&temp_path);
                return;
            }

            let down_url = prefix.to_string() + clip_clone.as_str();
            // println!("--> {}", down_url);

            let mut bytes = http_util::query_bytes_async(&down_url).await;
            let mut err_num = 1;
            while let Err(err) = bytes {
                println!("下载片段({})出错：{}, err_num={}", idx, err, err_num);
                if err_num >=5 {
                    err_clips.lock().unwrap().push(idx);
                    return;
                }
                // put_retry(&mut retry_num, &clone_pkg, clip_index, &clip);
                bytes = http_util::query_bytes_async(&down_url).await;
                err_num += 1;
            }
            println!("片段({})下载完成 len: {}", idx, bytes.as_ref().map(|op|op.len()).unwrap());
            //写入文件
            let origin_bytes;
            let result: &[u8] = if nd {
                let res = aes_util::decrypt(bytes.as_ref().unwrap(), &key, &iv);
                if let Ok(v) = res{
                    origin_bytes = v;
                    &origin_bytes
                }else{
                    println!("片段({}) Decode ERROR 解密过程出错：{}", idx, res.unwrap_err());
                    return;
                }
            } else {
                bytes.as_ref().unwrap()
            };
            if let Err(e) = write_file_async(result, &down_file_path).await{
                println!("写入片段[{}]失败， err={}", idx + 1, e);
                err_clips.lock().unwrap().push(idx);
            }else{
                config::add_prog(&temp_path);
            }
        });
        join_v.push(handler);
    }
    println!("join_v len = {}", join_v.len());
    // 存储AbortHandle
    let abort_v:Vec<AbortHandle> = join_v.iter()
            .map(|j|j.abort_handle())
            .collect();
    config::add_abort_handles(&temp_path, abort_v);

    let mut _idx = 1;
    for j in join_v{
        // println!("===> handler={} 开始执行",idx);
        let _ = j.await;
        // println!("===> handler={} 执行结束", idx);
        _idx += 1;
    }

    // 正常下载完成时设置标记为end
    if config::is_normal(&temp_path){
        let all_success = err_clips.lock().unwrap().is_empty();
        let mut msg = None;
        if !all_success {
            msg = Some(format!("以下片段出错没有下载完成: {:?}", err_clips.lock().unwrap()));
            println!("{}",msg.as_ref().unwrap());
            config::set_signal(&entity.temp_path, Signal::PartFinish, msg);
        }else{
            config::set_signal(&entity.temp_path, Signal::End, msg);
        }
        return all_success;
    }
   return false;
}
async fn exec_group(join_v: Vec<JoinHandle<()>>){
    for j in join_v.into_iter() {
        println!("===> handler= 开始执行");
        j.await;
        println!("===> handler= 执行结束");
    }
}

/// 构建文件名前缀
fn make_name(num: usize) -> String {
    if num < 1000 {
        let s = format!("{}", num);
        let pad = "0".repeat(4 - s.len()) + &s;

        return pad;
    }
    format!("{}", num)
}
///异步写入文件
async fn write_file_async(content: &[u8], path: &str) -> Result<(), Error> {
    let mut f = File::create(path).await?;

    // write_all() 会全部写入 ，write() 会写入部分
    f.write_all(content).await?;
    f.flush().await?;

    println!("写入成功 counter:{},content size:{} ", path, content.len());
    Ok(())
}
