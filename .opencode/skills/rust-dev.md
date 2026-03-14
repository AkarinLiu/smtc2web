# Rust Development Skill

## Commands

### Check Rust Code
```bash
cd src-tauri && cargo check
```
检查 Rust 代码是否有编译错误，不进行实际构建。

### Run Linter
```bash
cd src-tauri && cargo clippy
```
运行 Rust linter，检查代码风格和潜在问题。

### Format Code
```bash
cd src-tauri && cargo fmt
```
自动格式化 Rust 代码，遵循项目规范。

### Build Release
```bash
cd src-tauri && cargo build --release
```
构建发布版本的可执行文件。

### Run Tests
```bash
cd src-tauri && cargo test
```
运行所有测试（当前项目无测试）。

### Clean Build
```bash
cd src-tauri && cargo clean
```
清理构建产物。

## Project Structure
- `src-tauri/src/main.rs` - 程序入口
- `src-tauri/src/lib.rs` - 核心逻辑
- `src-tauri/Cargo.toml` - 依赖配置

## Key Patterns

### Error Handling
```rust
fn example() -> Result<T, Box<dyn std::error::Error>> {
    // 使用 ? 操作符传播错误
    let content = fs::read_to_string(&path)?;
    Ok(config)
}
```

### Logging
```rust
eprintln!("Error message");  // 用户可见错误
println!("Info message");     // 一般信息
```

### Windows API Async
```rust
// Windows API 异步调用模式
let result = async_operation()
    .and_then(|f| f.get());  // .get() 等待异步结果
```
