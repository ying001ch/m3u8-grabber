pub fn print_help(){
    println!(r#"m3u8-downloader v0.1.0

Usage: m3u8-downloader [URL]  [OPTIONS]
Options:
    --output=location       视频片段合并后的文件位置
    --file=""                  手动指定m3u8文件位置
    --combine=clip_dir      合并视频片段
    --combine_type=          合并类型 1-二进制合并 2-ffmpeg合并
    --temp="temp_path"      设置临时文件夹位置
    --proxy="proxy_url"     设置代理
    --H="key:v;k2:v2"       设置请求头,多个用;分隔
    --key="D2B"             设置解密key,16进制字符串
    --worker=16             设置下载并发数
    --noCombine             下载视频片段不合并
"#);
}