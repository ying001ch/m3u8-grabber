pub fn print_help(){
    println!(r#"m3u8-downloader v0.3.0

Usage: m3u8-downloader [URL]  [OPTIONS]
Options:
    --output=location       视频片段合并后的文件位置
    --file=""               手动指定m3u8文件位置
    --combine=clip_dir      合并视频片段
    --combine_type=         合并类型 1-二进制合并 2-ffmpeg合并
    --temp="temp_path"      设置临时文件夹位置
    --proxy="proxy_url"     设置代理,支持通过环境变量 proxy 设置
    --H="key:v ;; k2:v2"    设置请求头,多个用';;'分隔, 支持通过环境变量 headers/H 设置
    --key="D2B"             设置解密key,16进制字符串
    --worker=16             设置下载并发数, 默认16。支持通过环境变量 worker 设置
    --noCombine             下载视频片段不合并
Example:
    $env:proxy="http://127.0.0.1:7890"

    $env:worker=32

    m3u8-downloader "https://example.com/a1.m3u8" --output="/your/save/path"
"#);
}