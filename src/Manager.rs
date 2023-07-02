use anyhow::anyhow;
use anyhow::bail;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
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
use std::io::Error;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;

use std::thread;
use std::time::SystemTime;

/// 决定任务是 异步还是同步，合并文件还是下载文件
pub fn dispatch(param: DownParam, async_task: bool) -> Result<()>{
    //TODO 校验参数
    validate_param(&param)?;
    match param.task_type {
        //下载任务
        config::TASK_DOWN => run(param, async_task),
        //合并任务
        config::TASK_COM => if async_task{
                thread::spawn(move||combine::combine_clip(
                    param.combine_dir.unwrap().as_str(),
                    &param.save_path.as_str()).unwrap()
                    );
                    Ok(())
                }else{
                    combine::combine_clip(
                    param.combine_dir.unwrap().as_str(),
                    &param.save_path.as_str())
                },
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
    config::add_task(&entity)?; //使用片段临时路径 创建任务状态信息
    let one = move ||{
        let temp_path = entity.temp_path.clone();
        let save_path = entity.save_path.clone();
        let st = SystemTime::now(); //计时开始
        //手动创建了运行时，就可以不再使用main方法上的注解
        tokio::runtime::Runtime::new().unwrap()
                .block_on(download_async(entity));
        let spend_time = st.elapsed().unwrap().as_secs();

        println!("status is {:?}", config::get_status(&temp_path));
        if config::is_abort(&temp_path){
            println!("--->下载暂停");
            return ;
        }
        println!("下载完毕！总耗时：{}s no_combine:{}", spend_time, param.no_combine);

        //合并片段
        if !param.no_combine {
            combine::combine_clip(temp_path.as_str(), save_path.as_str()).unwrap();
        }
    };
    let handle = thread::spawn(one);
    if !async_task {
        handle.join().map_err(|e|anyhow!(format!("{:?}",e)))?;
    }
    Ok(())
}
///异步下载方法
async fn download_async(entity: M3u8Item::M3u8Entity){
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
            let permit = sem.acquire().await.unwrap();
            let down_file_path = format!("{}/{}.ts", temp_path_clone, make_name(idx as i32 +1));
            if tokio::fs::File::open(down_file_path.clone()).await.is_ok() {
                //文件已经存在，无需下载
                config::add_prog(&temp_path_clone);
                drop(permit);
                return;
            }

            let down_url = prefix_clone.to_string() + clip_clone.as_str();
            // println!("--> {}", down_url);

            let mut bytes = http_util::query_bytes_async(&down_url,0 as i32).await;
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
                err_clone.lock().unwrap().push(idx);
            }else{
                config::add_prog(&temp_path_clone);
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
    config::add_abort_handles(&temp_path, abort_v);

    let mut idx = 1;
    for j in join_v{
        // println!("===> handler={} 开始执行",idx);
        let _ = j.await;
        // println!("===> handler={} 执行结束", idx);
        idx += 1;
    }
    
    if err_clips.lock().unwrap().len() > 0{
        println!("以下片段出错没有下载完成: {:?}", err_clips.lock().unwrap());
        config::set_signal(&entity.temp_path, Signal::Exception);
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

/// 构建文件名前缀
fn make_name(num: i32) -> String {
    if num < 1000 {
        let s = format!("{}", num);
        let pad = "0".repeat(4 - s.len()) + &s;

        return pad;
    }
    format!("{}", num)
}
///异步写入文件
async fn write_file_async(result: &[u8], f_name: String) -> Result<(), Error> {
    let mut f = File::create(f_name.clone()).await?;

    let n = f.write(result).await?;
    f.flush().await?;

    println!("写入成功 counter:{}, size: {}bytes", f_name, n);
    Ok(())
}
