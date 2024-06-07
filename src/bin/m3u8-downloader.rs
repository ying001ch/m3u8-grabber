//!此二进制是纯命令行版本，不带GUI


use M3u8_Grabber::*;
use M3u8_Grabber::M3u8Item::DownParam;

fn main() {
    //判断是否使用命令行
    if use_cmd() {
        let param: DownParam = DownParam::from_cmd();
        Manager::dispatch(param, false).unwrap();
        return;
    }
    panic!("缺少参数")
}