use core::panic;
use std::{sync::RwLock,collections::HashMap, mem::discriminant};

use anyhow::{Result, anyhow, bail};
use lazy_static::lazy_static;
use tokio::{task::AbortHandle};
use serde::{Deserialize, Serialize};

use crate::{view::TaskView, M3u8Item::M3u8Entity, http_util};


/// 全局配置存储
/// 使用可变静态变量是不安全的，所以这里加了锁
static GLOBAL_CONFIG: RwLock<GlobalConfig> = RwLock::new(GlobalConfig{
    work_num: 16,
    proxy: None,
    combine_type: COMB_BIN,
});
lazy_static! {
    /// 任务集合
    static ref TASK_MAP:RwLock<HashMap<String,TaskState>> = RwLock::new(HashMap::new());
}

pub const TASK_DOWN: usize = 1; //下载视频
pub const TASK_COM: usize = 2;  //合并视频

pub const COMB_BIN: usize = 1; //二进制合并
pub const COMB_FFMPEG: usize = 2;  //ffmpeg合并视频

#[derive(Clone, Deserialize,Debug)]
pub struct GlobalConfig{
    pub work_num: usize,
    pub proxy: Option<String>,
    pub combine_type: usize,
}

#[derive(Debug, Clone, PartialEq, Default,Serialize)]
pub enum Signal {
    #[default] //设置枚举默认值
    Normal,
    Pause,
    PartFinish,
    End,
    Exception, //下载异常
}
#[derive(Debug, Default)]
struct TaskState{
    err_msg: String,
    hash: String,
    total: usize,
    finished: usize,
    state: Signal,
    file_name: String,
    abort_handles: Vec<AbortHandle>,
    meta: M3u8Entity,
}
impl TaskState {
    fn from(entity: &M3u8Entity)->Self{
        let mut task = Self::default();

        task.hash = entity.temp_path.to_owned();
        task.total = entity.clip_num();
        task.file_name = entity.save_path.to_owned();
        task.meta = entity.clone();

        task
    }
    fn progress(&self) -> f64{
        self.finished as f64 / self.total as f64
    }
}
//----------------------------------------------------------------
pub fn set_global_settings(config: &GlobalConfig){
    *GLOBAL_CONFIG.write().unwrap() = config.clone();
    http_util::update_client();
}
pub fn set_work_num(work_num: usize) {
    let a = GLOBAL_CONFIG.write();
    match a {
        Ok(mut res)=>res.work_num=work_num,
        Err(e)=>{
            log::error!("====> err: {}",e);
        }
    }
    // .unwrap();
    // a.borrow_mut().work_num = 12;
}
pub fn get_work_num() -> usize {
    GLOBAL_CONFIG.read().unwrap().work_num
}
//----------------------------------------------------------------
pub fn set_proxys(proxy_str: &str) {
    {
        let mut a = GLOBAL_CONFIG.write().unwrap();
        a.proxy = Some(proxy_str.to_owned());
    }
    http_util::update_client();
    log::info!("=========> 更新proxy成功： proxy:{}",proxy_str);
}
pub fn get_proxys() -> String {
    GLOBAL_CONFIG.read().unwrap().proxy.clone().unwrap_or("".to_string())
}
pub fn get_combine_type() -> usize {
    GLOBAL_CONFIG.read().unwrap().combine_type
}
//----------------------------------------------------------------
pub fn get_task_view() -> Vec<TaskView> {
    let guard = TASK_MAP.read().unwrap();
    let views:Vec<TaskView> = guard.values()
        .map(|f|{
            TaskView { task_id: f.hash.clone(), 
                err_msg: f.err_msg.clone(), // 获取错误信息
                status: f.state.clone(), // 获取任务状态
                progress: f.progress(),
                file_name: f.file_name.clone(),
                finished: f.finished,
                total: f.total,
            }
        })
        .collect();
    views
}
pub fn add_prog(task_hash: &str) {
    TASK_MAP.write().unwrap()
        .get_mut(task_hash)
        .map(|t|t.finished += 1);
}
//----------------------------------------------------------------
pub fn add_task(entity: &M3u8Entity) -> Result<()>{
    let mut guard = TASK_MAP.write().unwrap();
    let task_hash = &entity.temp_path;
    if let Some(t) = guard.get(task_hash){
        if let Signal::Normal = t.state {
            bail!("任务正在运行，无需添加")
        }
    }
    guard.insert(task_hash.to_string(), TaskState::from(entity));

    Ok(())
}
pub fn delete_task(task_hash: &str) -> Result<M3u8Entity>{
    if let Some(v) = TASK_MAP.write().unwrap().remove(task_hash){
        log::info!("task state is deleted. hash:{:?} fileName:{}",v.hash,v.file_name);
        Ok(v.meta)
    }else{
        bail!("任务不存在")
    }
}
pub fn get_meta(hash: &str)-> Option<M3u8Entity>{
    let guard = TASK_MAP.read().unwrap();
    guard.get(hash).map(|s|s.meta.clone())
}
pub fn abort_task(hash: &str)->Result<&str>{
    TASK_MAP.read().unwrap().get(hash)
        .map(|t|{
            t.abort_handles.iter()
                .filter(|h|!h.is_finished())
                .for_each(|h|h.abort());
            "暂停成功"
        }).ok_or(anyhow!("停止任务失败"))
}
pub fn add_abort_handles(task_hash:&str, handles: Vec<AbortHandle>){
    TASK_MAP.write().unwrap()
        .get_mut(task_hash)
        .map(|t|t.abort_handles = handles);
}
pub fn set_signal(task_hash: &str, ss: Signal, msg: Option<String>) {
    let mut guard = TASK_MAP.write().unwrap();
    guard.get_mut(task_hash)
        .map(|f|{
            f.state = ss;
            if let Some(msg) = msg{
                f.err_msg = msg;
            }
        });
}
pub fn is_end(task_hash: &str) -> bool{
    predict_status(task_hash, Signal::End)
}
pub fn is_abort(task_hash: &str) -> bool {
    predict_status(task_hash, Signal::Pause)
}
pub fn is_normal(task_hash: &str) -> bool {
    predict_status(task_hash, Signal::Normal)
}
pub fn get_status(task_hash: &str) -> Option<Signal> {
    TASK_MAP.read().unwrap().get(task_hash).map(|f|f.state.clone())
}
fn predict_status(task_hash: &str, signal: Signal) -> bool {
    TASK_MAP.read().unwrap().get(task_hash)
    .map(|f|{
            signal == f.state
        })
        .unwrap_or(false)
}