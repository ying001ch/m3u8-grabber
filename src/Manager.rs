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
    //设置代理 
    param.proxy.as_ref()
        .filter(|f|!f.is_empty())
        .inspect(|&p|config::set_proxys(p));
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
            log::info!("headers is :{:?}", v);
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

        log::info!("status is {:?}", config::get_status(temp_path));
        if config::is_abort(temp_path){
            log::info!("--->下载暂停");
            return ;
        }
        log::info!("下载完毕！总耗时：{}s no_combine:{} all_success:{}", spend_time, param.no_combine, all_success);

        //合并片段
        if all_success && !param.no_combine {
            let _ = combine::combine_clip(temp_path, save_path,param.combine_type, false)
                .inspect_err(|e|{
                    log::error!("合并片段出错：{}", e);
                });
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
    let clips = Arc::new(entity.media_play_list.segments.clone());
    let temp_path = &entity.temp_path;
    let nd = entity.need_decode();
    let key = entity.key;
    let iv = entity.iv;
    let multi_key = entity.multi_key();
    log::info!("multi_key:{}", multi_key);

    let prefix = entity.url_prefix.as_ref().unwrap();
    let mut join_v = vec![];
    let semaphore = Arc::new(Semaphore::new(config::get_work_num()));
    let err_vec: Vec<usize> = vec![];
    let err_clips = Arc::new(Mutex::new(err_vec));
    for idx in 0..clips.len() {
        let clips = Arc::clone(&clips);
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

            let clip_url = &clips[idx].uri;
            let down_url = if !clip_url.starts_with("http"){
                prefix.to_string() + clip_url
            }else{
                clip_url.to_string()
            };
            // println!("--> {}", down_url);
            let mut headers = vec![];
            if let Some(range) = clips[idx].byte_range.as_ref(){
                let offset = range.offset.unwrap_or(0);
                headers.push(("range", format!("bytes={}-{}", offset, offset+range.length - 1)));
            }

            let mut bytes = http_util::query_bytes_async(&down_url, Some(&headers)).await;
            let mut err_num = 1;
            while let Err(err) = bytes {
                log::error!("下载片段({})出错：{}, err_num={}", idx, err, err_num);
                if err_num >=5 {
                    err_clips.lock().unwrap().push(idx);
                    return;
                }
                // put_retry(&mut retry_num, &clone_pkg, clip_index, &clip);
                bytes = http_util::query_bytes_async(&down_url, Some(&headers)).await;
                err_num += 1;
            }
            log::info!("片段({})下载完成 len: {}", idx, bytes.as_ref().map(|op|op.len()).unwrap());
            //写入文件
            let origin_bytes;
            let result: &[u8] = if nd {
                // 如果每个key和 iv都不一样，那么就使用单独的key和iv
                let iv_own;
                let key_own;
                let mut k:&[u8] = &key;
                let mut iv:&[u8] = &iv;
                if multi_key{
                    let new_key = clips[idx].key.as_ref().unwrap();
                    let iv_s = new_key.iv.as_ref().unwrap();
                    log::debug!("new iv_s={}", iv_s);
                    iv_own = M3u8Item::hex2_byte(iv_s)
                        .expect(format!("解析片段iv 出错 iv:{}", iv_s).as_str());
                    iv = &iv_own;
                    //TODO key 
                    let key_uri = new_key.uri.as_ref().unwrap();
                    let key_res =  http_util::query_bytes_async::<&str,&str>(key_uri, None).await;
                    match key_res {
                        Ok(kb)=> {
                            log::debug!("new key bytes:{:?}", kb);
                            key_own = kb;
                            k = &key_own;
                        },
                        Err(err)=>{
                            log::error!("片段({}) query key_uri err:{}", idx, err);
                            return;
                        }
                    }
                }
                let res = aes_util::decrypt(bytes.as_ref().unwrap(), k, iv);
                if let Ok(v) = res{
                    origin_bytes = v;
                    &origin_bytes
                }else{
                    log::error!("片段({}) Decode ERROR 解密过程出错：{}", idx, res.unwrap_err());
                    return;
                }
            } else {
                bytes.as_ref().unwrap()
            };
            if let Err(e) = write_file_async(result, &down_file_path).await{
                log::error!("写入片段[{}]失败， err={}", idx + 1, e);
                err_clips.lock().unwrap().push(idx);
            }else{
                config::add_prog(&temp_path);
            }
        });
        join_v.push(handler);
    }
    log::info!("join_v len = {}", join_v.len());
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
            log::error!("{}",msg.as_ref().unwrap());
            config::set_signal(&entity.temp_path, Signal::PartFinish, msg);
        }else{
            config::set_signal(&entity.temp_path, Signal::End, msg);
        }
        return all_success;
    }
   return false;
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

    log::info!("写入成功 counter:{},content size:{} ", path, content.len());
    Ok(())
}
