# smtc2web

 一个基于 Rust 的 smtc2web 实现，用于在直播软件实时显示正在播放的歌曲。

![OBS 接入 smtc2web 截图](./screenshot.png)

## 下载

你可以从 GitHub Release 获取，下载后记得校验哈希值，防止软件被篡改。

## 编译

### 依赖

- Rust 工具链（建议使用 rustup 安装）
- Windows 10 以上操作系统

#### 主要依赖包：
- tokio 1.x (异步运行时，包含 rt-multi-thread 和 macros 特性)
- warp 0.3.x (Web 框架)
- serde 1.x (序列化/反序列化，包含 derive 特性)
- serde_json 1.x (JSON 处理)
- humantime 2.1.x (时间格式化)
- rust-embed 8.0.x (文件嵌入)
- mime_guess 2.0.x (MIME 类型猜测)

#### Windows 特定依赖：
- windows 0.58.x (Windows API 绑定，包含 Foundation、Media_Control、Win32_System_WinRT 特性)


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
