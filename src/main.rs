// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{vec};
use M3u8Item::{DownParam};

mod Manager;
mod http_util;
mod M3u8Item;
mod aes_util;
mod combine;
mod str_util;
mod config;
mod Test;
mod view;
mod command;

// #[tokio::main]
fn main() {
  config::init_task_view();
  //判断是否使用命令行
  if use_cmd(){
    let param:DownParam = DownParam::from_cmd();
    Manager::dispatch(param,false).unwrap();
    return;
  }
  //启动 Tauri GUI
  command::start_tauri();
}

fn use_cmd() -> bool {
  let args:Vec<String> = std::env::args().collect();
  args.len() > 1 && (args[1].starts_with("http") || args[1].contains("--combine"))
}