# smtc2web

 一个基于 Rust 的 smtc2web 实现，用于在直播软件实时显示正在播放的歌曲。

**LinuxDO 原帖子链接:** https://linux.do/t/topic/937994

**[参与翻译](https://zh.crowdin.com/project/smtc2web)** **[English](./README.en.md)**

![OBS 接入 smtc2web 截图](./screenshot.png)

> 新图标将 Bootstrap 的耳机图片和 Rust 的 Ferris 形象形成了戴耳机听音乐的图标。

<img src="./src-tauri/icons/icon.png" alt="图标" width="64" height="64">

## 推荐 IDE 配置

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- [TRAE](https://trae.com.cn/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- [Zed](https://zed.dev/)
- [OpenCode](https://opencode.ai)

## Linux MPRIS2 支持

smtc2web 现已支持 Linux 下通过 **MPRIS2**（Media Player Remote Interfacing Specification）协议获取正在播放的媒体信息。大多数 Linux 桌面环境中的本地媒体播放器（如 VLC、Rhythmbox、Audacious 等）均原生支持 MPRIS2，开箱即用。

### Snap / Flatpak 沙箱问题

通过 Snap 或 Flatpak 安装的播放器（如 Firefox、Spotify）默认受到沙箱限制，**无法被外部程序通过 D-Bus 访问**。你需要在终端中运行以下命令来放行 D-Bus 访问。

#### Snap

1. 查看当前已安装的 Snap 包：
   ```bash
   snap list
   ```

2. 安装并连接 `dbus` 接口（以 Firefox 为例）：
   ```bash
   sudo snap connect firefox:dbus-daemon
   ```
   > 如果 `dbus-daemon` 接口不可用，可尝试：
   > ```bash
   > sudo snap connect firefox:session-dbus-observing
   > ```

3. 验证接口已连接：
   ```bash
   snap connections firefox | grep dbus
   ```

4. **重启播放器**使配置生效。

#### Flatpak

1. 使用 Flatseal（图形界面，推荐）：
   ```bash
   flatpak install flathub com.github.tchx84.Flatseal
   ```
   打开 Flatseal → 选择目标播放器 → **系统总线（System Bus）** 或 **会话总线（Session Bus）** 中添加 `org.mpris.MediaPlayer2.*`。

2. 或通过命令行重写权限（以 Firefox 为例）：
   ```bash
   sudo flatpak override --socket=session-bus org.mozilla.firefox
   ```

3. **重启播放器**使配置生效。

### 排查其他 MPRIS2 问题

```bash
# 查看是否有 MPRIS2 播放器在运行
busctl --user list | grep mpris

# 手动查询播放器状态
gdbus call --session \
    --dest org.mpris.MediaPlayer2.<播放器名> \
    --object-path /org/mpris/MediaPlayer2 \
    --method org.freedesktop.DBus.Properties.Get \
    org.mpris.MediaPlayer2.Player PlaybackStatus

# 查看 smtc2web 日志（定位在 ~/.local/share/smtc2web/logs/）
```

### 已知限制

- **Snap Firefox**：部分版本 Snaps 的 AppArmor 策略非常严格，即使连接 `dbus` 接口也可能无法访问 MPRIS2。此情况建议使用系统包管理器直接安装 Firefox（`apt install firefox` 或 `dnf install firefox`）。
- **Flatpak**：部分 Flatpak 应用没有暴露 `org.mpris.MediaPlayer2.*` D-Bus 接口，需要应用开发者主动支持。

## 构建

- [Windows](https://smtc2web.org/wiki/compile/windows)
