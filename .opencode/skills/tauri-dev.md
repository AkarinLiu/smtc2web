# Tauri Development Skill

## Commands

### Development Mode
```bash
yarn tauri dev
```
启动 Tauri 开发模式，启用热重载。

### Production Build
```bash
yarn tauri build
```
构建生产版本，生成 .exe 安装包。

### Build Specific Target
```bash
yarn tauri build --target x86_64-pc-windows-msvc
yarn tauri build --target aarch64-pc-windows-msvc
```

### Check Configuration
```bash
cd src-tauri && cargo tauri inspect
```
检查 Tauri 配置。

## Key Files

### tauri.conf.json
- `app.windows` - 窗口配置（当前为空，使用无窗口模式）
- `app.withGlobalTauri` - 启用全局 Tauri 对象
- `bundle.targets` - 打包目标 (nsis, msi)
- `build.frontendDist` - 前端资源目录

### Cargo.toml
- `tauri` - 核心框架
- `tauri-plugin-*` - 官方插件
- `windows` - Windows API 依赖

## Frontend Communication

### Invoke Command (JS to Rust)
```javascript
const result = await window.__TAURI__.core.invoke("command_name", { arg: value });
```

### Listen Event (Rust to JS)
```javascript
await window.__TAURI__.event.listen("event_name", (event) => {
    console.log(event.payload);
});
```

## Adding New Plugin
1. Add dependency to `Cargo.toml`:
   ```toml
   tauri-plugin-name = "version"
   ```
2. Initialize in `lib.rs`:
   ```rust
   .plugin(tauri_plugin_name::init())
   ```
3. Configure in `tauri.conf.json` if needed.
