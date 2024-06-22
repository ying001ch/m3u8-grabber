use std::{env, io::{Read, Write}, process::{Command, Stdio}, string, thread};
use std::fs::ReadDir;

use anyhow::{Result, Context, bail};

use crate::config;

/// 调用FFMMPEG合并视频片段
pub fn combine_clip(clip_dir: &str, save_path: &str, comb_type: usize, async_task: bool) -> Result<()>{
    let dir_ex = std::fs::read_dir(clip_dir)
        .context(format!("clip_dir: {} not exists!", clip_dir))?;

    let save_path = get_output_name(save_path);
    println!("开始合并片段，cli_dir:{} save_path:{}", clip_dir, save_path);

    //判断使用二进制合并还是 ffmpeg
    if comb_type == config::COMB_BIN {
        return bin_combine(clip_dir, save_path, async_task);
    }

    // 1. 检测环境变量
    let ffmpeg_dir = std::env::var("FFMPEG_PATH")
        .context("没有配置 FFMPEG_PATH 环境变量")?;
    let ffmpeg = format!("{}/ffmpeg",ffmpeg_dir);
    println!("ffmpeg: {}", ffmpeg);

    // 2. 生成合并文件
    let com_file_name= build_com_file(clip_dir, dir_ex)?;
    println!("com_file_name: {}", &com_file_name);
   

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
        println!("===>output status={}", status);
        println!("===>output success={}", status.success());
        
        if status.success() {
            println!("开始删除临时文件:");
            std::fs::remove_dir_all(clip_dir).context("删除临时文件失败！")?;
            println!("删除临时文件完成！");
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

fn bin_combine(clip_dir: &str, save_path: String, async_task: bool) -> Result<(), anyhow::Error> {
    println!("将使用二进制合并！");

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
    let mut output_file = std::fs::File::create(&save_path).context("Failed to create the output file")?;

    let handler = move || {
        for video_file in video_files {
            let input_path = video_file.path();
            let mut input_file = std::fs::File::open(&input_path).context(format!("Failed to open file: {:?}", input_path))?;
            let mut buffer = [0; 1024];
            loop {
                let bytes_read = input_file.read(&mut buffer)?;
                if bytes_read == 0 {
                    break; // EOF
                }
                output_file.write_all(&buffer[..bytes_read])?;
            }
        }
        Ok::<(),anyhow::Error>(())
    };

   
    if async_task {
        thread::spawn(handler);
    }else{
        handler()?;
    }

    println!("Video files have been successfully combined into {}", save_path);
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
        println!("合并目录：{}为空", clip_dir);
        bail!("");
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

fn get_output_name(save_path: &str) -> String {
    if save_path.is_empty() || save_path.ends_with("/") || save_path.ends_with("\\") {
        return format!("{}output.ts", save_path);
    }
    save_path.to_string()
}