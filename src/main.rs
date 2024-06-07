// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{vec};
use M3u8_Grabber::*;
use M3u8Item::{DownParam};

mod Test;
mod command;

// #[tokio::main]
fn main() {
  //判断是否使用命令行
  if use_cmd(){
    let param:DownParam = DownParam::from_cmd();
    Manager::dispatch(param,false).unwrap();
    return;
  }
  //启动 Tauri GUI
  command::start_tauri();
}