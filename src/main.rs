// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use core::panic;
use serde::{Serialize, Deserialize};

mod Manager;
mod http_util;
mod M3u8Item;
mod aes_demo;
mod combine;
mod str_util;


fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![greet,submit_task])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[tauri::command]
fn greet(name: &str) -> String {
   format!("Hello, {}!", name)
}
#[derive(Serialize, Deserialize, Debug)]
pub struct DownParam {
    address: String,
    savePath: String,
    proxy: Option<String>,
    headers: Option<String>,
}
#[tauri::command]
fn submit_task(param_str: &str) -> &str{
  let param: DownParam = serde_json::from_str(param_str).unwrap();
  println!("deserialized = {:?}", param);
  return "chenggong==";

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
