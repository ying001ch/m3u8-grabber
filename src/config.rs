use std::{sync::Mutex, cell::RefCell};

struct GlobalConfig{
    work_num: usize,
    proxys: Option<String>,
    headers: Vec<(String,String)>,
}
//TODO 全局配置存储
static global_config: Mutex<GlobalConfig> = Mutex::new(GlobalConfig{
    work_num: 2,
    proxys: None,
    headers: vec![],
});
pub const TASK_DOWN: usize = 1; //下载视频
pub const TASK_COM: usize = 2;  //合并视频

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