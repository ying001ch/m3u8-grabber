# 0. 为什么要做

偶然间看到了 Rust 觉得这个语言挺有意思，就想着学来玩儿一下，做个东西练练手。

- 为什么要做M3u8下载器？

现在很多视频网站的视频都是m3u8的，想下载下来看，虽然市面上有很多很强大的下载器，如IDM 迅雷、XDM等，他们都做得很出色，但是因为种种原因，有些文件是主动不提供下载的。M3u8解析起来也比较简单，正好适合 魔域时间写着玩 也是作为练习吧！



# 1. 安装

```rust
cargo build --release
```

- 合并视频片段使用了 FFMPEG，需要事先下载好并配置环境变量 `FFMPEG_PATH`

```shell
FFMPEG_PATH="/ffmpeg/bin"
```



# 2. 使用

- 通过m3u8下载视频且合并片段

```shell
//   --output= 视频片段合并后的文件位置
./m3u8-downloader http://m3u8.address  --output=download_name.mp4
```

> 合并后的文件会放在在当前目录，名称可以不指定默认为 output.mp4

- 在某些情况下 通过地址无法直接下载m3u8文件，可以手动抓取m3u8内容保存成文件再下载

```shell
./m3u8-downloader http://m3u8.address --file ./m3u8_file_path
```

> 由于m3u8内容里面往往只有视频路径最后一截，所以即使有了m3u8文件还是要指定 m3u8地址

- 合并已存在的视频片段

```shell
// --output可省略 使用默认名称
./m3u8-downloader --combine ./video_clip_dir --output=download_name.mp4
```

> 会根据视频名称进行排序，只会添加 名称里包含 `.ts` 的文件

- 指定临时目录 以存放下载的视频片段，下载合并完毕之后会删除。不指定默认使用时间戳

```shell
// 任意位置添加参数
./m3u8-downloader http://m3u8.address --temp="temp_path"
```

- 使用 Http 代理

```shell
// 任意位置添加参数即可
--proxy=http://127.0.0.1:1081
```

- 设置Http Header

```shell
// 任意位置添加参数
--H="refer:https://yourAddress" --H="origin: http://yourOrigin"
```

- 设置key

```shell
# 任意位置添加参数，字符串形式
--key="D2BAfb82c3GAf4EA"
```

- 设置下载任务的线程数

```shell
--worker=4  // 手动指定线程数，默认为4
```

# 3. 下载失败

由于网络等原因，有时会下载失败。可以重新下载，只要指定上一次下载所使用的临时目录，已经下载好的文件会跳过，不会重复下载