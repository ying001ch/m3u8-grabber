use core::panic;
use std::{sync::{Mutex, MutexGuard}, cell::RefCell, collections::HashMap, mem::discriminant};

use tokio::{runtime::Runtime, task::AbortHandle};

pub struct GlobalConfig{
    work_num: usize,
    proxys: Option<String>,
    headers: Vec<(String,String)>,
    signal: Signal,
}
//TODO 全局配置存储
static global_config: Mutex<GlobalConfig> = Mutex::new(GlobalConfig{
    work_num: 2,
    proxys: None,
    headers: vec![],
    signal: Signal::Normal,
});
// static rt: Mutex<Option<Runtime>> = Mutex::new(None);
pub const TASK_DOWN: usize = 1; //下载视频
pub const TASK_COM: usize = 2;  //合并视频

#[derive(Debug, Clone)]
pub enum Signal {
    Normal,
    Pause,
    End,
}
struct TaskState{
    hash: String,
    state: Signal,
    abort_handles: Vec<AbortHandle>,
    headers: Vec<(String,String)>,
}
static task_map:Mutex<Option<HashMap<String,TaskState>>> = Mutex::new(None);

//----------------------------------------------------------------
pub fn set_work_num(work_num: usize) {
    let a = global_config.lock();
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
    global_config.lock().unwrap().work_num
}
//----------------------------------------------------------------
pub fn set_proxys(ss: String) {
    let mut a = global_config.lock().unwrap();
    a.proxys = Some(ss);
}
pub fn get_proxys() -> String {
    global_config.lock().unwrap().proxys.clone().unwrap_or("".to_string())
}
//----------------------------------------------------------------
pub fn set_headers(v: Vec<(String,String)>) {
    let mut a = global_config.lock().unwrap();
    a.headers = v;
}
pub fn get_headers() -> Vec<(String,String)> {
     global_config.lock().unwrap().headers.clone()
}
static PROP:Mutex<Option<Prog>> = Mutex::new(None);
pub fn init_progress(total_num: usize){
    println!("===>初始化进度 : {}", total_num);
    *PROP.lock().unwrap() = Some(Prog{
        total: total_num,
        finished: 0,
        status: 1,
    });
}
pub fn get_progress() -> (usize,usize,i32) {
    let gard: MutexGuard<'_, Option<Prog>> = PROP.lock().unwrap();
    //获取MutexGuard后 as_ref()获取 只读引用
    let po = gard.as_ref();
    match po {
        Some(p)=> (p.total, p.finished,p.status),
        None=>(0,0,0),
    }
    
}
pub fn add_prog() {
    let mut ga = PROP.lock().unwrap();
    //获取MutexGuard后 as_mut()获取 写引用
    let a = ga.as_mut().unwrap();
    a.finished += 1;
    if a.finished > a.total {
        a.finished =a.total;
    }
    //需要改变 Optiona内部的值时 使用map
    // ga.as_mut().map(|f|{
    //     f.finished += 1;
    // });
}
struct Prog{
    total: usize,
    finished: usize,
    status: i32 //0-未开始 1-进行中 -1-异常退出
}
//----------------------------------------------------------------
pub fn add_task(task_hash: &str) {
    let mut guard = task_map.lock().unwrap();
    if guard.is_none(){
        println!("任务状态没有初始化");
        *guard = Some(HashMap::new());
    }
    let map = guard.as_mut().unwrap();
    map.insert(task_hash.to_string(), TaskState { 
        hash: task_hash.to_string(),
        state: Signal::Normal, 
        abort_handles: vec![],
        headers: vec![]
    });
}
pub fn set_signal(task_hash: &str, ss: Signal) {
    let mut guard = task_map.lock().unwrap();
    if guard.is_none(){
        panic!("任务状态没有初始化");
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
    task_map.lock().unwrap().as_ref().unwrap().get(task_hash).map(|f|f.state.clone())
}
fn predict_status(task_hash: &str, signal: Signal) -> bool {
    task_map.lock().unwrap().as_ref().unwrap().get(task_hash)
    .map(|f|{
            discriminant(&signal) == discriminant(&f.state)
        })
        .unwrap_or(true)
}