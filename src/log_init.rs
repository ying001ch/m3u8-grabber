use std::{env, path::Path, str::FromStr};

use anyhow::anyhow;
use log::LevelFilter;
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};

use crate::use_cmd;

const LOG_PATTERN: &str = "[{d(%Y-%m-%dT%H:%M:%S%.3f)} {h({l}):<5.5} {T} {M}] {m}{n}";
const CONFIG_PATH: &str = "log4rs.yaml";
const LOG_FILE: &str = "log/running.log";
const DEFAULT_LEVEL : LevelFilter = LevelFilter::Info;

/// 初始化日志组件
/// 1. 如果使用命令行参数，则只输出到控制台
/// 2. 如果存在配置文件，则使用配置文件初始化日志组件
/// 3. 否则，输出到控制台和文件
pub fn run() {
    let result:anyhow::Result<_> = if use_cmd() {
        println!("使用命令行参数初始化日志组件");
        let config = Config::builder().appender(
            console_appender(),
        );
        let level = env::var("rust_log")
            .ok()
            .and_then(|e|LevelFilter::from_str(e.as_str()).ok())
            .unwrap_or(DEFAULT_LEVEL);
        println!("日志级别: {level}");
        let root = Root::builder()
            .appenders(["console"])
            .build(level);
        log4rs::init_config(config.build(root).unwrap()).map(|_|())
            .map_err(|e|anyhow!("初始化日志组件异常 {e}"))
    } else if Path::new(CONFIG_PATH).exists(){
        log4rs::init_file(CONFIG_PATH, Default::default())
            .map_err(|e|anyhow!("初始化日志组件异常 {e}"))
    }else{
        let config = Config::builder().appenders(
            [console_appender(),file_appender(LOG_FILE)]
        );
        let root = Root::builder()
            .appenders(["console","file"])
            .build(DEFAULT_LEVEL);
        log4rs::init_config(config.build(root).unwrap())
            .map(|_|())
            .map_err(|e|anyhow!("初始化日志组件异常 {e}"))
    };
    if let Err(e) = result {
        eprintln!("{:?}", e);
    };
}

fn console_appender() -> Appender {
    Appender::builder().build(
        "console",
        Box::new(
            ConsoleAppender::builder()
                .encoder(Box::new(PatternEncoder::new(LOG_PATTERN)))
                .build(),
        ),
    )
}
fn file_appender<P : AsRef<Path>>(p: P) -> Appender {
    Appender::builder().build(
        "file",
        Box::new(
            FileAppender::builder()
                .encoder(Box::new(PatternEncoder::new(LOG_PATTERN)))
                .build(p).unwrap(),
        ),
    )
}
