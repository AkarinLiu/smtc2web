# Windows API Usage Skill

## Project Windows Features

在 `Cargo.toml` 中定义:
```toml
[dependencies.windows]
features = [
  "Foundation",
  "Foundation_Collections",
  "Media_Control",       # SMTC 媒体控制
  "Storage",             # 文件存储
  "Storage_Streams",     # 流操作
  "Win32_System_WinRT", # WinRT 支持
  "Win32_UI_WindowsAndMessaging",
  "Win32_UI_Shell",
  "Win32_System_Console",
  "Win32_System_Threading",
  "Win32_Foundation",
  "Win32_Security",
]
```

## SMTC (System Media Transport Controls)

### Get Session Manager
```rust
use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;

let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
    .and_then(|f| f.get())?;
```

### Get Current Session
```rust
let session = manager.GetCurrentSession()?;
```

### Get Media Properties
```rust
let info = session.TryGetMediaPropertiesAsync()
    .and_then(|f| f.get())?;

let title = info.Title()?;
let artist = info.Artist()?;
let album = info.AlbumTitle()?;
```

### Get Playback Status
```rust
use windows::Media::Control::GlobalSystemMediaTransportControlsSessionPlaybackStatus;

let playback_info = session.GetPlaybackInfo()?;
let is_playing = playback_info.PlaybackStatus()? 
    == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing;
```

### Get Timeline (Position/Duration)
```rust
let timeline = session.GetTimelineProperties()?;
let position = timeline.Position()?.Duration;  // 100-nanosecond intervals
let duration = timeline.EndTime()?.Duration;
```

## Async Pattern

Windows WinRT APIs 使用异步模式:
```rust
// 正确模式
async_operation()
    .and_then(|f| f.get())  // 等待异步完成

// 不要直接 await，Rust 没有 .await 语法
```

## Thumbnail/Album Art

```rust
use windows::Storage::Streams::{Buffer, DataReader};

let thumbnail = info.Thumbnail()?;
let stream = thumbnail.OpenReadAsync().and_then(|f| f.get())?;
let buffer = Buffer::Create(size)?;
let result = stream.ReadAsync(&buffer, size, InputStreamOptions::ReadAhead).get()?;
let reader = DataReader::FromBuffer(&result)?;
let length = result.Length()? as usize;
let mut data = vec![0u8; length];
reader.ReadBytes(&mut data)?;
```

## Common Patterns

### Error Handling
```rust
if let Ok(value) = windows_api_call() {
    // 成功处理
} else {
    eprintln!("Windows API call failed");
}
```

### Duration Conversion
```rust
// Windows 时间为 100 纳秒单位
let seconds = duration / 10_000_000;
let milliseconds = duration / 10_000;
```
