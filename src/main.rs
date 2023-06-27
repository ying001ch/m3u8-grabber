// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use core::panic;
use std::{thread, ops::{DerefMut, Deref}, borrow::Borrow, future, time::Duration, error::Error, fmt::{Display, Write}, vec};
use M3u8Item::{DownParam, M3u8Entity};
use anyhow::{Result, bail, anyhow};
use config::Signal;
use serde_json::{Value::{Number as ValNumber, self, String as jString}, Map};
use serde_json::Number as Number;
use view::TaskView;

mod Manager;
mod http_util;
mod M3u8Item;
mod aes_util;
mod combine;
mod str_util;
mod config;
mod Test;
mod view;

// #[tokio::main]
fn main() {
  //判断是否使用命令行
  if use_cmd(){
    let param:DownParam = DownParam::from_cmd();
    dispatch(param,false).unwrap();
    return;
  }
  //启动图形界面
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![submit_task, combine,pause,get_progress ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

fn use_cmd() -> bool {
  let args:Vec<String> = std::env::args().collect();
  args.len() > 1 && (args[1].starts_with("http") || args[1].contains("--combine"))
}

#[tauri::command]
fn submit_task(param_str: &str) -> Result<&str, String>{
  println!("raw str: {}",param_str);
  let param: DownParam = serde_json::from_str(param_str).unwrap();
  println!("deserialized = {:?}", param);

  dispatch(param,true).map(|_|"提交成功").map_err(|e|e.to_string())
}
#[tauri::command]
fn combine(param_str: &str) -> Result<&str, String>{
  let param: DownParam = serde_json::from_str(param_str).unwrap();
  println!("combine deserialized = {:?}", param);
  dispatch(param,true).map(|_|"合并任务提交成功！").map_err(|e|e.to_string())
}
#[tauri::command]
fn pause(task_hash: &str) -> &'static str{
  config::set_signal(task_hash, Signal::Pause);
  return "暂停信号已发出";
}
/// TODO 修改成获取状态 TaskView
#[tauri::command]
fn get_progress() -> Vec<TaskView>{
  // let mut map = serde_json::Map::with_capacity(2);
  // let (total, finished,status) = config::get_progress();
  // println!("=====> total = {}, finished = {}, status = {}", total, finished, status);
  // map.insert("status".to_string(), ValNumber(Number::from(status)));
  // if status == -1 {
  //   // map.insert("status".to_string(), ValNumber(Number::from(-1)));
  // }else if status == 1 {
  //   let prog = format!("{:.4}", finished as f64/total as f64);
  //   map.insert("progress".to_string(), jString(prog));
  // }else{
  //   map.insert("progress".to_string(), ValNumber(Number::from(0)));
  // }
  //TODO 刷新 任务状态
  return vec![];
}
fn dispatch(param: DownParam, async_task: bool) -> Result<()>{
  //TODO 校验参数
  validate_param(&param)?;
  match param.task_type {
    //下载任务
    config::TASK_DOWN => Manager::run(param, async_task),
    //合并任务
    config::TASK_COM => if async_task{
              thread::spawn(move||combine::combine_clip(
                  param.combine_dir.unwrap().as_str(),
                  &param.save_path.as_str())
                );
                Ok(())
              }else{
                combine::combine_clip(
                  param.combine_dir.unwrap().as_str(),
                  &param.save_path.as_str())
              },
      _=> bail!("任务类型不对"),
  }

}
fn validate_param(param: &DownParam)-> Result<()>{
  //校验 合并参数、下载参数
  match param.task_type {
    config::TASK_COM => {
      param.combine_dir.as_ref().ok_or(anyhow!("合并目录未指定")).map(|_|())
    },
    config::TASK_DOWN=>{
      if param.address.is_empty() || !param.address.starts_with("http"){
        bail!("下载地址格式不正确")
      }
      Ok(())
    },
    _=> bail!("任务类型不对")
  }
}