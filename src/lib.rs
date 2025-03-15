pub mod Manager;
mod http_util;
pub mod M3u8Item;
mod aes_util;
mod combine;
pub mod config;
pub mod view;
pub mod help;
mod async_runtime;
mod db;
pub mod log_init;

pub fn use_cmd() -> bool {
    let args:Vec<String> = std::env::args().collect();
    args.len() > 1 && (args[1].starts_with("http") || args[1].contains("--combine"))
}