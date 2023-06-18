use std::{env, io::Write, process::Command, string};

pub fn combine_clip(clip_dir: &str, save_path: &str) {
    let save_path = get_output_name(save_path);
    println!("开始合并片段，cli_dir:{} save_path:{}", clip_dir, save_path);
    // 1. 检测环境变量
    let ffmpeg_dir = std::env::var("FFMPEG_PATH")
    .expect("没有配置 FFMPEG_PATH 环境变量");
    let ffmpeg = format!("{}/ffmpeg",ffmpeg_dir);
    println!("ffmpeg: {}", ffmpeg);

    // 2. 生成合并文件
    let com_file_name={
        let com_file_name = format!("{}/combine.txt",clip_dir);
        let mut com_txt = std::fs::File::create(&com_file_name)
                .expect("创建合并文件失败");
        let mut file_list = vec![];
        for entry in  std::fs::read_dir(clip_dir).expect("读取文件夹失败"){
            let file_name = entry.unwrap().file_name().into_string()
                    .expect("获取文件名时错误");
            if !file_name.contains(".ts") {
                continue;
            }
            let line = format!("file '{}'\n", file_name);
            file_list.push(line);
        }
        file_list.sort_by(|x,y|{
            x.cmp(y)
        });
        for f in file_list {
            com_txt.write_all(f.as_bytes())
                    .expect(format!("生成合并文件时出错，file:{}", com_file_name).as_str());
        }
        com_txt.flush().unwrap();
        com_file_name
    };
    println!("com_file_name: {}", &com_file_name);
   

    let output_name = save_path;
    // 3.调用合并
    let output = 
        Command::new(ffmpeg)
                .arg("-f").arg("concat").arg("-i")
                .arg(com_file_name.as_str()).arg("-c").arg("copy")
                .arg(output_name)
                .output()
                .expect("ffmpeg exec error!");
    
    let output_str = String::from_utf8_lossy(&output.stderr);
    println!("output_str={}", output_str);
    println!("开始删除临时文件:");
    
    std::fs::remove_dir_all(clip_dir).expect("删除临时文件失败！");
    println!("删除临时文件完成！");
}

fn get_output_name(save_path: &str) -> String {
    if(save_path.is_empty() || save_path.ends_with("/") || save_path.ends_with("\\")){
        return format!("{}output.mp4", save_path);
    }
    save_path.to_string()
}