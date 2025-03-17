use std::sync::LazyLock;

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

static USE_CMD_STATE: LazyLock<bool> = LazyLock::new(||{
    let args:Vec<String> = std::env::args().collect();
    args.len() > 1 && (args[1].starts_with("http") || args[1].contains("--combine"))
});

pub fn use_cmd() -> bool {
    USE_CMD_STATE.clone()
}