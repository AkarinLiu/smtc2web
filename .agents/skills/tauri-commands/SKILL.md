---
name: tauri-commands-extension
description: Tauri 命令扩展指南，包括命令定义、注册和前端集成
license: MIT
compatibility: Tauri v1/v2, smtc2web >=1.0
metadata:
  category: api-development
  language: zh-CN
  framework: tauri
  type: skill
---

# Tauri Commands Extension Skill

## Adding New Command

### Step 1: Define Command in Rust

在 `lib.rs` 或单独模块中添加:

```rust
#[tauri::command]
pub fn my_command(arg: String) -> Result<String, String> {
    // 业务逻辑
    Ok(format!("Result: {}", arg))
}
```

### Step 2: Register Command

在 `run()` 函数中注册:

```rust
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![my_command])
    .run(tauri::generate_context!())
    .expect("error");
```

### Step 3: Call from Frontend

```javascript
const result = await window.__TAURI__.core.invoke("my_command", { arg: "value" });
```

## API Endpoint Pattern

### Adding New HTTP API

在 `lib.rs` 中添加新的 warp 路由:

```rust
// 定义新的端点
let new_api = warp::path!("api" / "new_endpoint")
    .and(with_state(state))
    .map(|s: Shared| {
        // 获取数据
        warp::reply::json(&response)
    });

// 注册到服务器
warp::serve(api.or(new_api).or(static_files))
    .run((address, port))
    .await;
```

## Data Types

### Song Struct Pattern
```rust
#[derive(Default, Clone, Serialize, PartialEq)]
struct Song {
    title: String,
    artist: String,
    album: String,
    album_art: Option<String>,  // base64 编码图片
    position: Option<String>,   // "MM:SS" 格式
    duration: Option<String>,
    pct: Option<f64>,
    is_playing: bool,
    last_update: u64,
}
```

### Response Types
- 使用 `Serialize` derive 导出到 JSON
- 使用 `Option<T>` 表示可选字段
- 使用 `String` 而非 `&str` 确保所有权

## Frontend Integration

### Reactive State (Vue)
```javascript
import { createApp, reactive } from "./lib/js/vue.esm-browser.js";

const state = reactive({
    title: "",
    artist: "",
    albumArt: null,
    isPlaying: false
});

async function poll() {
    try {
        const data = await window.__TAURI__.core.invoke("get_song_data");
        Object.assign(state, data);
    } catch (e) {
        console.error(e);
    }
    setTimeout(poll, 100);
}
```

### Error Handling
```javascript
try {
    const result = await window.__TAURI__.core.invoke("command");
} catch (error) {
    console.error("Command failed:", error);
}
```
