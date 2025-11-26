use std::{io::{BufWriter, Read, Write}, path::{Path, PathBuf}, process::{Command, Stdio}, thread, vec};
use std::fs::ReadDir;

use anyhow::{anyhow, bail, Context, Result};

use crate::config;

/// 调用FFMMPEG合并视频片段
pub fn combine_clip(clip_dir: &str, save_path: &str, comb_type: usize, async_task: bool) -> Result<()>{
    let dir_ex = std::fs::read_dir(clip_dir)
        .context(format!("clip_dir: {} not exists!", clip_dir))?;

    let save_path = get_output_path(save_path);
    log::info!("开始合并片段，cli_dir:{} save_path:{}", clip_dir, save_path.to_string_lossy());

    //判断使用二进制合并还是 ffmpeg
    if comb_type == config::COMB_BIN {
        return bin_combine(clip_dir, save_path.as_ref(), async_task);
    }

    // 1. 优先使用配置的目录，如果未配置则使用环境变量
    let ffmpeg_dir = if let Some(dir) = config::get_ffmpeg_dir() {
        dir
    } else {
        std::env::var("FFMPEG_PATH")
            .context("没有配置 FFMPEG_PATH 环境变量")?
    };
    let ffmpeg = format!("{}/ffmpeg",ffmpeg_dir);
    log::info!("ffmpeg: {}", ffmpeg);

    // 2. 生成合并文件
    let com_file_name= build_com_file(clip_dir, dir_ex)?;
    log::info!("com_file_name: {}", &com_file_name);
   

    let output_name = save_path;
    // 3.调用合并
    let mut child = 
        Command::new(ffmpeg)
                .arg("-f").arg("concat").arg("-i")
                .arg(com_file_name.as_str()).arg("-c").arg("copy")
                .arg(output_name)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .stdin(Stdio::piped())
                .spawn()
                .expect("ffmpeg exec error!");
    let mut stdin = child.stdin.take().expect("stdin take err");
    std::thread::spawn(move ||{
        stdin.write_all(b"y\r\n").expect("写入失败");
    });
    
    let clip_dir = clip_dir.to_owned();
    let child_listener = move ||{
        let status = child.wait()?;
        log::info!("===>output status={}", status);
        log::info!("===>output success={}", status.success());
        
        if status.success() {
            log::info!("开始删除临时文件:");
            std::fs::remove_dir_all(clip_dir).context("删除临时文件失败！")?;
            log::info!("删除临时文件完成！");
        }
        Ok::<(),anyhow::Error>(())
    };
    if async_task {
        std::thread::spawn(child_listener);
        Ok(())
    }else{
        child_listener()
    }
}

fn bin_combine(clip_dir: &str, save_path: &Path, async_task: bool) -> Result<(), anyhow::Error> {
    log::info!("将使用二进制合并！");

    // 获取所有视频文件
    let video_files: Vec<_> = std::fs::read_dir(clip_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| is_video_file(entry))
        .collect();

    // 检查是否有视频文件
    if video_files.is_empty() {
        bail!("No video files found in the specified directory.");
    }

    // 合并文件（简化处理，实际可能需要使用特定库）
    if !save_path.parent().ok_or_else(||anyhow!("save_path parent now exists"))?.exists(){
        std::fs::create_dir_all(save_path.parent().unwrap()).context("创建输出目录失败")?;
    }
    let output_file = std::fs::File::create(save_path).context("5")?;    

    let cd = clip_dir.to_string();
    let handler = move || {
        let mut buf_writer = BufWriter::new(output_file);
        for video_file in video_files {
            let input_path = video_file.path();
            let mut input_file = std::fs::File::open(&input_path).context(format!("Failed to open file: {:?}", input_path))?;
            let mut buffer = [0; 1024*4];
            loop {
                let bytes_read = input_file.read(&mut buffer)?;
                if bytes_read == 0 {
                    break; // EOF
                }
                buf_writer.write_all(&buffer[..bytes_read])?;
            }
        }
        let _ = buf_writer.flush().inspect_err(|e|{
            log::error!("flush err: {}",e);
        });
        log::info!("开始删除临时文件:");
        std::fs::remove_dir_all(cd).context("删除临时文件失败！")?;
        log::info!("删除临时文件完成！");
        
        Ok::<(),anyhow::Error>(())
    };

   
    if async_task {
        thread::spawn(handler);
    }else{
        handler()?;
    }

    log::info!("Video files have been successfully combined into {:?}", save_path);
    Ok(())
}

fn is_video_file(entry: &std::fs::DirEntry) -> bool {
    let path = entry.path();
    if let Some(ext) = path.extension() {
        ext.to_string_lossy().eq_ignore_ascii_case("ts") // 示例中仅检查mp4文件，根据需要扩展
    } else {
        false
    }
}

/// 构建合并描述文件
fn build_com_file(clip_dir: &str, dir_ex: ReadDir) -> Result<String> {
    let com_file_name = format!("{}/combine.txt", clip_dir);
    let mut com_txt = std::fs::File::create(&com_file_name)
        .context("创建合并文件失败")?;
    let mut file_list = vec![];
    for entry in dir_ex {
        let file_name = entry.unwrap().file_name().into_string()
            .expect("获取文件名时错误");
        if !file_name.contains(".ts") {
            continue;
        }
        let line = format!("file '{}'\n", file_name);
        file_list.push(line);
    }
    if file_list.is_empty() {
        log::error!("合并目录：{}为空", clip_dir);
        bail!("合并目录为空");
    }
    file_list.sort_by(|x, y| {
        x.cmp(y)
    });
    for f in file_list {
        com_txt.write_all(f.as_bytes())
            .context(format!("生成合并文件时出错，file:{}", com_file_name))?;
    }
    com_txt.flush()?;
    Ok(com_file_name)
}

fn get_output_path(save_path: &str) -> PathBuf {
    let mut save_path_buf = PathBuf::from(save_path);
    format_path(&mut save_path_buf);
    if save_path.is_empty() || save_path_buf.is_dir() {
        save_path_buf.push("output.ts");
    }
    save_path_buf
}
fn format_path(save_path: &mut PathBuf) {
    let mut temp = save_path.as_path();
    let mut v = vec![];
    v.push(save_path.file_name().unwrap().to_string_lossy().to_string());
    loop {
        let p = temp.parent();
        if p.is_none() || p.unwrap().to_string_lossy().is_empty() {
            break;
        }
        let p = p.unwrap();
        let new_path = p.file_name().unwrap().to_str().unwrap().trim();
        v.push(new_path.to_string());

        temp = p;
    }
    v.reverse();
        
    *save_path = PathBuf::from_iter(v);
    println!("new save_path: {:?}", save_path);
}