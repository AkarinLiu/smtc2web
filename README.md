# smtc2web

 一个基于 Rust 的 smtc2web 实现，用于在直播软件实时显示正在播放的歌曲。

![OBS 接入 smtc2web 截图](./screenshot.png)

## 需求

- 操作系统：Windows 10 以上，建议使用 Windows 11
- 软件： Rustup

## 下载

你可以从 GitHub Release 获取，下载后记得校验哈希值，防止软件被篡改。

## 编译

执行以下命令进行编译：

```powershell
cargo build --release
```

编译成功后会在 `target/release` 目录下出现二进制文件。

## 使用方法

### OBS

1. 点击添加源按钮，选择浏览器；
2. 在 URL 输入以下网址：http://localhost:3030；
3. 点击确定保存。

### 哔哩哔哩直播姬

1. 点击添加素材按钮，选择浏览器；
2. 在 URL 输入以下网址：http://localhost:3030；
3. 点击确定保存。

其他软件大同小异，不再赘述。
