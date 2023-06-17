// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use core::panic;
use std::{thread, ops::{DerefMut, Deref}, borrow::Borrow};
use M3u8Item::{DownParam, M3u8Entity};

mod Manager;
mod http_util;
mod M3u8Item;
mod aes_demo;
mod combine;
mod str_util;
mod config;


fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![greet,submit_task, combine])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[tauri::command]
fn greet(name: &str) -> String {
   format!("Hello, {}!", name)
}
#[tauri::command]
fn submit_task(param_str: &str) -> Result<&str,&str>{
  println!("raw str: {}",param_str);
  let param: DownParam = serde_json::from_str(param_str).unwrap();
  println!("deserialized = {:?}", param);

  dispatch(param);
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
  dispatch(param);
  return "合并任务提交成功！";
}
const TASK_DOWN: usize = 1; //下载视频
const TASK_COM: usize = 2;  //合并视频
fn dispatch(param: DownParam){
  match param.task_type {
    TASK_DOWN => {
              thread::spawn(move||Manager::run(param));
              },
    TASK_COM => {
              thread::spawn(move||combine::combine_clip(param.combine_dir.unwrap().as_str(),
                &param.save_path.as_str()));
              },
      _=> println!("任务类型不对"),
  }

}