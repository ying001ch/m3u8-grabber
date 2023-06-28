use core::panic;
use std::{sync::{RwLock}, cell::RefCell, collections::HashMap, mem::discriminant, default::{self}};

use tokio::{runtime::Runtime, task::AbortHandle};

use crate::view::TaskView;

pub struct GlobalConfig{
    work_num: usize,
    proxys: Option<String>,
    headers: Vec<(String,String)>,
    signal: Signal,
}
//TODO 全局配置存储
static global_config: RwLock<GlobalConfig> = RwLock::new(GlobalConfig{
    work_num: 2,
    proxys: None,
    headers: vec![],
    signal: Signal::Normal,
});
pub const TASK_DOWN: usize = 1; //下载视频
pub const TASK_COM: usize = 2;  //合并视频

#[derive(Debug, Clone, Default)]
pub enum Signal {
    #[default] 
    Normal,
    Pause,
    End,
}
#[derive(Debug, Default)]
struct TaskState{
    hash: String,
    total: usize,
    finished: usize,
    state: Signal,
    abort_handles: Vec<AbortHandle>,
    headers: Vec<(String,String)>,
}
impl TaskState {
    fn new(hash: &str, total: usize) -> Self{
        let mut task = Self::default();
        task.hash = hash.to_owned();
        task.total = total;

        task
    }
    fn progress(&self) -> f64{
        self.finished as f64 / self.total as f64
    }
}
static task_map:RwLock<Option<HashMap<String,TaskState>>> = RwLock::new(None);

//----------------------------------------------------------------
pub fn set_work_num(work_num: usize) {
    let a = global_config.write();
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
    global_config.read().unwrap().work_num
}
//----------------------------------------------------------------
pub fn set_proxys(ss: String) {
    let mut a = global_config.write().unwrap();
    a.proxys = Some(ss);
}
pub fn get_proxys() -> String {
    global_config.read().unwrap().proxys.clone().unwrap_or("".to_string())
}
//----------------------------------------------------------------
pub fn set_headers(v: Vec<(String,String)>) {
    let mut a = global_config.write().unwrap();
    a.headers = v;
}
pub fn get_headers() -> Vec<(String,String)> {
     global_config.read().unwrap().headers.clone()
}
pub fn get_task_view() -> Vec<TaskView> {
    let guard = task_map.read().unwrap();
    if guard.is_none(){
        return vec![];
    }
    let views:Vec<TaskView> = guard.as_ref().unwrap().values()
        .map(|f|{
            TaskView { task_id: f.hash.clone(), 
                err_msg: "".to_string(),
                status: "".to_string(), 
                progress: f.progress() 
            }
        })
        .collect();
    views
}
pub fn add_prog(task_hash: &str) {
    task_map.write().unwrap().as_mut().unwrap()
        .get_mut(task_hash)
        .map(|t|t.finished += 1);
}
struct Prog{
    total: usize,
    finished: usize,
    status: i32 //0-未开始 1-进行中 -1-异常退出
}
//----------------------------------------------------------------
pub fn init_task_view(){
    *task_map.write().unwrap() = Some(HashMap::new());
}
pub fn add_task(task_hash: &str, clip_num: usize) {
    let mut guard = task_map.write().unwrap();
    if guard.is_none(){
        panic!("=====taskView没有初始化!=====");
    }
    guard.as_mut().unwrap()
        .insert(task_hash.to_string(), TaskState::new(task_hash, clip_num));
}
pub fn set_signal(task_hash: &str, ss: Signal) {
    let mut guard = task_map.write().unwrap();
    if guard.is_none(){
        panic!("=====taskView没有初始化!=====");
    }
    let map = guard.as_mut().unwrap()
        .get_mut("k")
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
    task_map.read().unwrap().as_ref().unwrap().get(task_hash).map(|f|f.state.clone())
}
fn predict_status(task_hash: &str, signal: Signal) -> bool {
    task_map.read().unwrap().as_ref().unwrap().get(task_hash)
    .map(|f|{
            discriminant(&signal) == discriminant(&f.state)
        })
        .unwrap_or(true)
}