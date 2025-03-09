//!此二进制是纯命令行版本，不带GUI


use M3u8_Grabber::*;
use M3u8_Grabber::M3u8Item::DownParam;

fn main() {
    log_init::run();
    //判断是否使用命令行
    if use_cmd() {
        let param: DownParam = DownParam::from_cmd();
        let _ = Manager::dispatch(param, false).inspect_err(|e|{
            log::error!("dispatch task error : {}", e);
        });
        return;
    }
    help::print_help();
}