# 元数据后端

阅读上级 README 后开发。Rust、Axum、SQLx 与 PostgreSQL；独立 Cargo 工程和锁文件。

`src/api.rs` 负责 HTTP，`src/auth.rs` 负责管理员与会话，`src/protocol.rs` 校验壁纸和发布 JSON，`src/probe.rs` 只流式校验外部资源、不持久化媒体，`migrations` 负责本服务独立数据库。`src/main.rs` 为运行入口。

执行 `cargo test`、`cargo run`、`cargo build --release`。测试用独立 `XIAOBAN_TEST_DATABASE_URL` 和唯一数据库 schema，不访问生产数据或外部媒体。默认 `HOST=127.0.0.1`、`PORT=55006`；支持 `DATABASE_URL`、`PUBLIC_ORIGIN`、`SECURE_COOKIE`、`FRONTEND_DIST`，具体以服务配置为准。
