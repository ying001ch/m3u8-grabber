// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use core::panic;
use std::{thread, ops::{DerefMut, Deref}, borrow::Borrow, future, time::Duration};
use M3u8Item::{DownParam, M3u8Entity};

mod Manager;
mod http_util;
mod M3u8Item;
mod aes_demo;
mod combine;
mod str_util;
mod config;

// #[tokio::main]
fn main() {
  //判断是否使用命令行
  if useCmd(){
    let param:DownParam = DownParam::from_cmd();
    dispatch(param,false);
    return;
  }
  //启动图形界面
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![submit_task, combine])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

fn useCmd() -> bool {
  let args:Vec<String> = std::env::args().collect();
  args.len() > 1 && (args[1].starts_with("http") || args[1].contains("--combine"))
}

#[tauri::command]
fn submit_task(param_str: &str) -> Result<&str,&str>{
  println!("raw str: {}",param_str);
  let param: DownParam = serde_json::from_str(param_str).unwrap();
  println!("deserialized = {:?}", param);

  dispatch(param,true);
  return Ok("任务提交成功！");

  // let args:Vec<String> = std::env::args().collect();
  // if args[1] == "--combine"{
  //     if args.len() < 3{
  //         panic!("合并操作还需要指定片段存放目录；m3u8-downloader --combine /clip_path");
  //     }
  //     combine::combine_clip(&args[2]);
  //     return "";
  // }
  
  // Manager::run();
}
#[tauri::command]
fn combine(param_str: &str) -> &str{
  let param: DownParam = serde_json::from_str(param_str).unwrap();
  println!("combine deserialized = {:?}", param);
  dispatch(param,true);
  return "合并任务提交成功！";
}
fn dispatch(param: DownParam, async_task: bool){
  match param.task_type {
    config::TASK_DOWN => if async_task{
                thread::spawn(move||Manager::run(param));
              }else{
                Manager::run(param);
              },
    config::TASK_COM => if async_task{
              thread::spawn(move||combine::combine_clip(
                  param.combine_dir.unwrap().as_str(),
                  &param.save_path.as_str())
                );
              }else{
                combine::combine_clip(
                  param.combine_dir.unwrap().as_str(),
                  &param.save_path.as_str())
              },
      _=> println!("任务类型不对"),
  }

}