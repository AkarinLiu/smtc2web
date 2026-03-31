---
name: debugging-logging
description: 调试和日志技巧，包括控制台控制、日志输出和常见问题排查
license: MIT
compatibility: smtc2web >=1.0
metadata:
  category: debugging
  language: zh-CN
  type: skill
---

# Debugging & Logging Skill

## Console Window Control

### Show Console
在 `config.toml` 中设置:
```toml
show_console = true
```
程序启动时显示控制台窗口。

### Hide Console
```toml
show_console = false
```
程序启动时隐藏控制台窗口（默认行为）。

### Manual Control (Rust)
```rust
use crate::console;

// 隐藏控制台
console::hide_console();

// 显示控制台（调试用）
console::show_console();
```

## Logging

### Print Statements
```rust
println!("Variable: {}", value);
eprintln!("Error: {}", error);
```

### File Logging
当前项目使用 `eprintln!` 输出到控制台。如需文件日志，可添加:
```rust
// 添加到 Cargo.toml
log = "0.4"
env_logger = "0.11"
```

## Debug Mode

### Enable Debug Logging
```bash
# 设置环境变量
$env:RUST_LOG="debug"
yarn tauri dev
```

### View Running Output
1. 临时设置 `show_console = true` 在配置中
2. 重新启动应用
3. 查看控制台输出

## Common Issues

### Window Not Showing
- 检查 `tauri.conf.json` 中 `app.windows` 配置
- 本项目使用无窗口模式，仅托盘运行

### Server Not Starting
- 检查端口是否被占用 (默认 3030)
- 查看配置中 `server_port` 值

### SMTC Not Working
- 确保 Windows Media Player 或支持 SMTC 的应用正在播放
- 检查防火墙设置
