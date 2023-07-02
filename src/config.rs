use core::panic;
use std::{sync::RwLock,collections::HashMap, mem::discriminant};

use anyhow::{Result, anyhow, bail};
use tokio::{task::AbortHandle};
use serde::Serialize;

use crate::{view::TaskView, M3u8Item::M3u8Entity};


pub struct GlobalConfig{
    work_num: usize,
    proxys: Option<String>,
    headers: Vec<(String,String)>,
}
//TODO 全局配置存储
static GLOBAL_CONFIG: RwLock<GlobalConfig> = RwLock::new(GlobalConfig{
    work_num: 2,
    proxys: None,
    headers: vec![],
});
pub const TASK_DOWN: usize = 1; //下载视频
pub const TASK_COM: usize = 2;  //合并视频

#[derive(Debug, Clone, Default,Serialize)]
pub enum Signal {
    #[default] //设置枚举默认值
    Normal,
    Pause,
    End,
    Exception, //下载异常
}
#[derive(Debug, Default)]
struct TaskState{
    hash: String,
    total: usize,
    finished: usize,
    state: Signal,
    file_name: String,
    abort_handles: Vec<AbortHandle>,
    headers: Vec<(String,String)>,
}
impl TaskState {
    fn new(hash: &str, total: usize, file_name: &str) -> Self{
        let mut task = Self::default();
        task.hash = hash.to_owned();
        task.total = total;
        task.file_name = file_name.to_owned();

        task
    }
    fn progress(&self) -> f64{
        self.finished as f64 / self.total as f64
    }
}
static TASK_MAP:RwLock<Option<HashMap<String,TaskState>>> = RwLock::new(None);

//----------------------------------------------------------------
pub fn set_work_num(work_num: usize) {
    let a = GLOBAL_CONFIG.write();
    match a {
        Ok(mut res)=>res.work_num=work_num,
        Err(e)=>{
            println!("====> err: {}",e);
        }
    }
    // .unwrap();
    // a.borrow_mut().work_num = 12;
}
pub fn get_work_num() -> usize {
    GLOBAL_CONFIG.read().unwrap().work_num
}
//----------------------------------------------------------------
pub fn set_proxys(ss: String) {
    let mut a = GLOBAL_CONFIG.write().unwrap();
    a.proxys = Some(ss);
}
pub fn get_proxys() -> String {
    GLOBAL_CONFIG.read().unwrap().proxys.clone().unwrap_or("".to_string())
}
//----------------------------------------------------------------
pub fn set_headers(v: Vec<(String,String)>) {
    let mut a = GLOBAL_CONFIG.write().unwrap();
    a.headers = v;
}
pub fn get_headers() -> Vec<(String,String)> {
     GLOBAL_CONFIG.read().unwrap().headers.clone()
}
pub fn get_task_view() -> Vec<TaskView> {
    let guard = TASK_MAP.read().unwrap();
    if guard.is_none(){
        return vec![];
    }
    let views:Vec<TaskView> = guard.as_ref().unwrap().values()
        .map(|f|{
            TaskView { task_id: f.hash.clone(), 
                err_msg: "".to_string(), // TODO 获取错误信息
                status: f.state.clone(), //TODO 获取任务状态
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
    TASK_MAP.write().unwrap().as_mut().unwrap()
        .get_mut(task_hash)
        .map(|t|t.finished += 1);
}
//----------------------------------------------------------------
pub fn init_task_view(){
    *TASK_MAP.write().unwrap() = Some(HashMap::new());
}
pub fn add_task(entity: &M3u8Entity) -> Result<()>{
    let mut guard = TASK_MAP.write().unwrap();
    if guard.is_none(){
        panic!("=====taskView没有初始化!=====");
    }
    let task_hash = &entity.temp_path;
    let clip_num = entity.clip_num();
    if let Some(t) = guard.as_mut().unwrap().get(task_hash){
        if let Signal::Normal = t.state {
            bail!("任务正在运行，无需添加")
        }
    }
    guard.as_mut().unwrap()
        .insert(task_hash.to_string(), TaskState::new(task_hash, clip_num, &entity.save_path));
    Ok(())
}
pub fn abort_task(hash: &str)->Result<&str>{
    TASK_MAP.read().unwrap().as_ref().unwrap().get(hash)
        .map(|t|{
            t.abort_handles.iter()
                .filter(|h|!h.is_finished())
                .for_each(|h|h.abort());
            "暂停成功"
        }).ok_or(anyhow!("停止任务失败"))
}
pub fn add_abort_handles(task_hash:&str, handles: Vec<AbortHandle>){
    TASK_MAP.write().unwrap().as_mut().unwrap()
        .get_mut(task_hash)
        .map(|t|t.abort_handles = handles);
}
pub fn set_signal(task_hash: &str, ss: Signal) {
    let mut guard = TASK_MAP.write().unwrap();
    if guard.is_none(){
        panic!("=====taskView没有初始化!=====");
    }
    guard.as_mut().unwrap()
        .get_mut(task_hash)
        .map(|f|f.state = ss);
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
    TASK_MAP.read().unwrap().as_ref().unwrap().get(task_hash).map(|f|f.state.clone())
}
fn predict_status(task_hash: &str, signal: Signal) -> bool {
    TASK_MAP.read().unwrap().as_ref().unwrap().get(task_hash)
    .map(|f|{
            discriminant(&signal) == discriminant(&f.state)
        })
        .unwrap_or(true)
}