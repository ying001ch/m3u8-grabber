/// tauri命令集合


use anyhow::{Result};

use crate::M3u8Item::{DownParam, M3u8Entity};
use crate::config::{self, Signal};
use crate::view::TaskView;
use crate::Manager;

pub fn start_tauri(){
    //启动图形界面
  tauri::Builder::default()
  .invoke_handler(tauri::generate_handler![
      submit_task, 
      combine_cmd,
      pause,
      get_progress,
  ])
  .run(tauri::generate_context!())
  .expect("error while running tauri application");
}

/// 提交视频下载任务
#[tauri::command]
pub fn submit_task(param_str: &str) -> Result<&str, String>{
    println!("raw str: {}",param_str);
    let param: DownParam = serde_json::from_str(param_str).unwrap();
    println!("deserialized = {:?}", param);

    Manager::dispatch(param,true).map(|_|"提交成功").map_err(|e|e.to_string())
}
/// 合并视频片段
#[tauri::command]
pub fn combine_cmd(param_str: &str) -> Result<&str, String>{
    let param: DownParam = serde_json::from_str(param_str).unwrap();
    println!("combine deserialized = {:?}", param);
    Manager::dispatch(param,true).map(|_|"合并任务提交成功！").map_err(|e|e.to_string())
}
/// 暂停任务
#[tauri::command]
pub fn pause(task_hash: &str) -> &'static str{
    config::set_signal(task_hash, Signal::Pause);
    return "暂停信号已发出";
}
/// 获取任务状态
/// TODO 修改成获取状态 TaskView
#[tauri::command]
pub fn get_progress() -> Vec<TaskView>{
    //TODO 刷新 任务状态
    return config::get_task_view();
}