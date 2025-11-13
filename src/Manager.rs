use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use bytes::Bytes;
use m3u8_rs::Key;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;
use tokio::task::AbortHandle;
use anyhow::Result;

use crate::aes_util;
use crate::async_runtime;
use crate::combine;
use crate::config::GlobalConfig;
use crate::config::Signal;
use crate::http_util;
use crate::M3u8Item;
use crate::M3u8Item::DownParam;
use crate::config;
use std::io::Error;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use std::time::SystemTime;

/// 决定任务是 异步还是同步，合并文件还是下载文件
pub fn dispatch(param: DownParam, async_task: bool) -> Result<()>{
    // 校验参数
    validate_param(&param)?;
    if !async_task {
        config::set_global_settings(&GlobalConfig{
            work_num: param.worker_num,
            proxy: param.proxy.clone(),
            combine_type: param.combine_type,
            last_save_dir: None
        }, false);
    }
    match param.task_type {
        //下载任务
        config::TASK_DOWN => {
            let entity = M3u8Item::M3u8Entity::from(&param)?;
            config::add_task(&entity)?; //使用片段临时路径 创建任务状态信息
            run(entity, async_task)
        },
        //合并任务
        config::TASK_COM => combine::combine_clip(
            param.combine_dir.unwrap().as_str(),
            &param.save_path.as_str(),
            config::get_combine_type(),
            async_task),
        _=> bail!("任务类型不对"),
    }
}
pub fn resume_task(task_hash: &str)-> Result<&str>{
    match config::get_status(task_hash) {
        Some(signal) => {
            if signal == Signal::Normal{
                bail!("任务已经在运行中");
            }
            if signal == Signal::End{
                bail!("任务已完成");
            }
            if let Some(entity) = config::get_meta(task_hash){
                run(entity, true).map(|_|"操作成功")
            }else {
                bail!("没有找到任务");
            }
        }, 
        None => {
            bail!("没有找到任务")
        }
    }
}
pub fn delete_task(task_hash: &str) -> Result<&str>{
    if let Some(signal) = config::get_status(task_hash){
        if signal == Signal::Normal{
            config::abort_task(task_hash)?;
        }
        // 删除临时文件
        let entity = config::delete_task(task_hash)?;
        if Path::new(&entity.temp_path).exists(){
            if std::fs::remove_dir_all(entity.temp_path.as_str()).is_err(){
                log::warn!("删除临时文件失败:{}", entity.temp_path);
            }
        }
        log::info!("删除任务完成！");
        Ok("操作成功")
    }else {
        bail!("没有找到任务")
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
fn run(entity: M3u8Item::M3u8Entity, async_task: bool) -> Result<()>{
    config::set_signal(&entity.temp_path, Signal::Normal,None);
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
        log::info!("下载完毕！总耗时：{}s no_combine:{} all_success:{}", spend_time, entity.no_combine, all_success);

        //合并片段
        if all_success && !entity.no_combine {
            let _ = combine::combine_clip(temp_path, save_path,config::get_combine_type(), false)
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
    config::clear_prog(&temp_path);
    let nd = entity.need_decode();
    let key = entity.key;
    let iv = entity.iv;
    let multi_key = entity.multi_key();
    let headers = Arc::new(entity.headers.clone());
    log::info!("multi_key:{}", multi_key);

    let prefix = entity.url_prefix.as_ref().unwrap();
    let root_prefix = entity.root_prefix.as_ref().unwrap();
    let mut join_v = vec![];
    let semaphore = Arc::new(Semaphore::new(config::get_work_num()));
    let err_vec: Vec<usize> = vec![];
    let err_clips = Arc::new(Mutex::new(err_vec));
    for idx in 0..clips.len() {
        let clips = Arc::clone(&clips);
        // let clip_clone = clip.clone();
        let prefix = prefix.to_string();
        let root_prefix = root_prefix.to_string();
        let temp_path = temp_path.clone();
        let sem = semaphore.clone();
        let headers = Arc::clone(&headers);
        let err_clips = Arc::clone(&err_clips);
        let handler = tokio::spawn(async move{
            let _permit = sem.acquire().await.unwrap();
            let down_file_path = format!("{}/{}.ts", temp_path, make_name(idx +1));
            if tokio::fs::File::open(down_file_path.as_str()).await.is_ok() {
                //文件已经存在，无需下载
                config::add_prog(&temp_path);
                return;
            }

            // 拼接下载地址
            let clip_url = &clips[idx].uri;
            let down_url = if clip_url.starts_with("/"){
                root_prefix.to_string() + clip_url
            } else if clip_url.starts_with("http"){
                clip_url.to_string() 
            }else{
                prefix.to_string() + clip_url
            };
            // 设置请求头
            let mut real_headers = vec![];
            real_headers.extend_from_slice(&headers);
            if let Some(range) = clips[idx].byte_range.as_ref(){
                let offset = range.offset.unwrap_or(0);
                real_headers.push(("range".to_owned(), format!("bytes={}-{}", offset, offset+range.length - 1)));
            }

            let mut bytes = http_util::query_bytes_async(&down_url, Some(&real_headers)).await;
            //出错充实5次
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
            //解密文件
            let origin_bytes;
            let result: &[u8] = if nd {
                // 如果每个key和 iv都不一样，那么就使用单独的key和iv
                let mut iv_own = iv;
                let mut key_own = key;
                if multi_key{
                    match get_new_key_iv(clips[idx].key.as_ref().unwrap(), idx).await{
                        Ok((key, iv)) => {
                            key_own.clone_from_slice(&key);
                            iv_own = iv;
                            assert_eq!(*key.slice(0..key.len()), key_own);
                        },
                        Err(_) => return,
                    }
                }
                log::debug!("iv_own:{:?} key_own:{:?}", iv_own, &key_own);
                let res = aes_util::decrypt(bytes.as_ref().unwrap(), &key_own, &iv_own);
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
            config::set_signal_async(&entity.temp_path, Signal::PartFinish, msg).await;
        }else{
            config::set_signal_async(&entity.temp_path, Signal::End, msg).await;
        }
        return all_success;
    }
   return false;
}

// 在 aes_util 模块中添加此函数
pub async fn get_new_key_iv(key: &Key, idx: usize) -> Result<(Bytes, [u8;16])> {
    let iv_s = key.iv.as_ref().unwrap();
    log::debug!("new iv_s={}", iv_s);
    let iv_own = M3u8Item::hex2_byte(iv_s)
        .expect(format!("解析片段iv 出错 iv:{}", iv_s).as_str());
    //TODO key 
    let key_uri = key.uri.as_ref().unwrap();
    let key_res =  http_util::query_bytes_async::<&str,&str>(key_uri, None).await;
    match key_res {
        Ok(kb)=> {
            log::debug!("new key bytes:{:?} length={}", kb, kb.len());
            return Ok((kb, iv_own));
        },
        Err(err)=>{
            log::error!("片段({}) query key_uri err:{}", idx, err);
            bail!("片段({}) query key_uri err:{}", idx, err);
        }
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

    log::info!("写入成功 counter:{},content size:{} ", path, content.len());
    Ok(())
}
