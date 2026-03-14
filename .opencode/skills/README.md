# smtc2web Skills 索引

## Rust & Tauri 开发 Skills

| Skill | 描述 |
|-------|------|
| [rust-dev](rust-dev.md) | Rust 基础开发命令和模式 |
| [tauri-dev](tauri-dev.md) | Tauri 开发命令和配置 |
| [debug-logging](debug-logging.md) | 调试和日志技巧 |
| [windows-api](windows-api.md) | Windows API 使用指南 |
| [tauri-commands](tauri-commands.md) | 扩展 Tauri 命令和 API |
| [system-integration](system-integration.md) | 系统集成功能 |

## 快速开始

### 开发
```bash
yarn tauri dev
```

### 构建
```bash
yarn tauri build
```

### 代码检查
```bash
cd src-tauri && cargo check
cd src-tauri && cargo clippy
cd src-tauri && cargo fmt
```

## 文档结构

- `src-tauri/src/lib.rs` - 核心逻辑
- `src-tauri/src/config.rs` - 配置管理
- `src-tauri/src/tray.rs` - 系统托盘
- `src-tauri/src/theme.rs` - 主题文件托管
- `src-tauri/src/console.rs` - 控制台控制
