//! tauri命令集合


use anyhow::Result;
use tauri::Manager as tauri_manager;
use M3u8_Grabber::config::GlobalConfig;

use crate::M3u8Item::DownParam;
use crate::config::{self, Signal};
use crate::view::TaskView;
use crate::Manager;

pub fn start_tauri(){
    //启动图形界面 
  tauri::Builder::default()
  .plugin(tauri_plugin_dialog::init())
  .setup(|app| {
    let window = app.get_webview_window("main").unwrap();
    // 生产环境禁用右键菜单
    if !cfg!(debug_assertions) {
      window.eval(&format!("window.addEventListener('contextmenu', e => e.preventDefault());"))?;
    }
    // 初始化配置
    let config = config::load_global_settings();
    config::set_global_settings(&config, false);
    Ok(())
  })
  .invoke_handler(tauri::generate_handler![
      submit_task, 
      combine_cmd,
      pause,
      get_progress,
      save_settings,
      resume,
      delete_task,
      load_settings,
  ])
  .run(tauri::generate_context!())
  .expect("error while running tauri application");
}

/// 提交视频下载任务
#[tauri::command]
pub fn submit_task(param_str: &str) -> Result<&str, String>{
    log::debug!("raw str: {}",param_str);
    let param: DownParam = serde_json::from_str(param_str)
        .map_err(|e| format!("参数解析失败: {}", e))?;
    log::info!("deserialized = {:?}", param);

    // 记住最近一次的保存目录（取父目录）
    if !param.save_path.is_empty() && !param.save_path.ends_with("\\") && !param.save_path.ends_with("/") {
        let save_dir = std::path::Path::new(&param.save_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or(param.save_path.clone());
        config::set_last_save_dir(&save_dir, true);
    }

    Manager::dispatch(param,true).map(|_|"提交成功").map_err(|e|e.to_string())
}
/// 合并视频片段
#[tauri::command]
pub fn combine_cmd(param_str: &str) -> Result<&str, String>{
    let param: DownParam = serde_json::from_str(param_str)
        .map_err(|e| format!("参数解析失败: {}", e))?;
    println!("combine deserialized = {:?}", param);
    Manager::dispatch(param,true).map(|_|"合并任务提交成功！").map_err(|e|e.to_string())
}
/// 暂停任务
#[tauri::command]
pub fn pause(task_hash: &str) -> Result<&str,String>{
    config::set_signal(task_hash, Signal::Pause,None);
    log::info!("set signal success");
    return config::abort_task(task_hash).map_err(|e|e.to_string());
}
#[tauri::command]
pub fn resume(task_hash: &str) -> Result<&str,String>{
    return Manager::resume_task(task_hash).map_err(|e|e.to_string());
}
#[tauri::command]
pub fn delete_task(task_hash: Vec<String>) -> Result<Vec<String>,String>{
    let mut success_task = vec![];
    for hash in task_hash.iter(){
        if let Err(e) = Manager::delete_task(hash) {
            return Err(e.to_string());
        }
        success_task.push(hash.to_string());
    }
    return Ok(success_task);
}
/// 修改成获取状态 TaskView
#[tauri::command]
pub fn get_progress(load_db: bool) -> Vec<TaskView>{
    // 刷新 任务状态
    return config::get_task_view(load_db);
}
#[tauri::command]
pub fn save_settings(config: GlobalConfig) -> Result<&'static str,String>{
    log::info!("save_settings: {:?}",&config);
    config::set_global_settings(&config, true);
    return Ok("保存成功");
}
#[tauri::command]
pub fn load_settings() -> GlobalConfig{
    config::load_global_settings()
}