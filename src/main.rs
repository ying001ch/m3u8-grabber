// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use M3u8_Grabber::*;
use M3u8Item::DownParam;

mod gui;

// #[tokio::main]
fn main() {
  log_init::run();
  //判断是否使用命令行
  if use_cmd(){
    let param:DownParam = DownParam::from_cmd();
    Manager::dispatch(param,false).inspect_err(|e|{
        log::error!("dispatch task error : {}", e);
    }).unwrap();
    return;
  }
  //启动 Tauri GUI
  gui::command::start_tauri();
}