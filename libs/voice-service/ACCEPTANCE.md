# 交付与验收记录

日期：2026-10-28（Asia/Shanghai）。环境：Windows x64、Rust / Cargo 1.94.1。

## 修改文件

仅新增 `libs/voice-service/`，修改 `libs/README.md` 一行索引；没有创建 workflow、修改其他子项目或提交/推送 Git。工作期间仓库另有其他改动，未参与处理。

| 文件（相对本目录） | 内容 |
| --- | --- |
| Cargo.toml、Cargo.lock、.gitignore | 独立 Rust 2021 crate、可选 stdio、锁定依赖、排除构建产物 |
| src/lib.rs、src/backends.rs | 公共导出及全部注入 trait / 类型，公共项有文档，禁止 unsafe |
| src/server.rs、src/security.rs | 启停、路由、请求限制、鉴权、Host / Origin、同意缓存与截止 |
| src/audio.rs | WAV / MP3 / FLAC / OGG 解码、流式重采样、低能量切段、WAV 编码 |
| src/openai.rs | 转写、朗读及音色列表 |
| src/mcp.rs | 新旧 MCP、四个设备工具、并发 / 截止 / 后台提示 |
| src/bin/xiaoban-voice-mcp-stdio.rs | 有界、并行的 stdio 到本服务 HTTP 转发 |
| tests/openai.rs、tests/mcp.rs、tests/logging.rs | 假后端的真实 HTTP / stdio 集成测试、脱敏日志断言 |
| tests/common/mod.rs、tests/fixtures.rs | 假实现、请求与音频辅助函数 |
| tests/generate_fixtures.py、tests/fixtures/*.b64 | 自生成小音频的可重现源码及三份 Base64 文本样本 |
| examples/serve_mock.rs | 可直接 curl 的假服务，无真实设备，输出短音而非朗读 |
| README.md、DEPENDENCIES.md、ACCEPTANCE.md | 用法、接口、安全、接入、限制、许可和验收原始输出 |

## 逐条验收

| 任务单标准 | 实际结果 |
| --- | --- |
| 1. cargo test | **通过**，22 项测试（库单测 1、二进制单测 1、日志 1、MCP 8、OpenAI 11），0 失败、0 忽略。原始输出附后 |
| 2. cargo clippy -- -D warnings | **通过**；最终使用更广的 `--all-targets -- -D warnings`，覆盖示例、二进制和测试，输出附后 |
| 3. 示例服务与三条 curl | **通过**：转写 200、MP3 200 / 2342 字节、不带令牌 401；MP3 另经 ffmpeg 解码退出码 0。命令及实际输出如下 |
| 4. README | **已提供**：接口、安全、依赖与许可、后端接入、已知限制、验证命令、官方 MCP 规范来源 |
| 5. 隔离和产物 | **符合范围**：无 Tauri / GTK / 系统音频设备依赖；库不落盘、无出站调用。HTTP 服务需 tokio / axum 网络监听，另行请求的 stdio 二进制需回环 HTTP 客户端。可关闭默认 stdio feature，纯库编译通过。待纳入版本控制的音频仅小型文本样本，无 exe / mp3 / wav 等二进制文件 |

### curl 复现

以下命令在仓库根目录执行；令牌是公开的**测试值**，产品接入须自行生成随机秘密。

先启动服务：

```powershell
$env:XIAOBAN_VOICE_TOKEN = 'curl-demo-token-at-least-24-characters'
$env:XIAOBAN_VOICE_BIND = '127.0.0.1:58139'
cargo run --manifest-path libs/voice-service/Cargo.toml --example serve_mock
```

实际启动输出：

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.25s
     Running `libs\voice-service\target\debug\examples\serve_mock.exe`
http://127.0.0.1:58139 (mock consents automatically; speech is a 440 Hz tone)
```

另一个终端生成 0.25 秒 / 16 kHz / 单声道 PCM16 WAV 及朗读请求文件（均写入忽略的 target）：

```powershell
@'
import wave, math, struct, json
from pathlib import Path
p = Path('libs/voice-service/target')
with wave.open(str(p/'curl-input.wav'), 'wb') as f:
    f.setparams((1, 2, 16000, 0, 'NONE', 'NONE'))
    f.writeframes(b''.join(struct.pack('<h', round(8000*math.sin(2*math.pi*440*i/16000))) for i in range(4000)))
(p/'curl-speech.json').write_text(json.dumps({'model':'anything','input':'hello'}), encoding='utf-8')
'@ | python -
```

三条实际请求：

```powershell
curl.exe -sS -w '\nHTTP %{http_code}\n' -H 'Authorization: Bearer curl-demo-token-at-least-24-characters' -F 'file=@libs/voice-service/target/curl-input.wav' -F 'model=anything' http://127.0.0.1:58139/v1/audio/transcriptions
curl.exe -sS -w 'HTTP %{http_code}; Content-Type %{content_type}; downloaded %{size_download} bytes\n' -H 'Authorization: Bearer curl-demo-token-at-least-24-characters' -H 'Content-Type: application/json' --data-binary '@libs/voice-service/target/curl-speech.json' -o libs/voice-service/target/curl-speech.mp3 http://127.0.0.1:58139/v1/audio/speech
curl.exe -sS -w '\nHTTP %{http_code}\n' http://127.0.0.1:58139/v1/audio/voices
```

实际输出：

```text
{"text":"你好，小伴"}
HTTP 200
HTTP 200; Content-Type audio/mpeg; downloaded 2342 bytes
{"error":{"code":"unauthorized","message":"unauthorized","type":"invalid_request_error"}}
HTTP 401
```

MP3 使用本机 ffmpeg 7.1 执行 `ffmpeg -hide_banner -loglevel error -i libs/voice-service/target/curl-speech.mp3 -f null -`，无解码错误，进程退出码 0；另有库内 MP3→WAV→PCM 集成断言。没有进行扬声器听音。示例服务已在验收后停止。

## 取舍、确认事项与未测范围

- 任务委托人已明确允许 **Unicode-3.0 传递依赖**；其余依赖选择 MIT / Apache-2.0 / BSD 分支，许可快照记录 123 个依赖包。为避免 MPL，使用 hound / claxon / lewton / puremp3，不使用推荐的 Symphonia。
- 重采样选择流式线性插值以保持低内存；已断言 44.1→16 kHz 的正弦频率、时长、波形误差和立体声均值。没有抗混叠滤波，高频输入可能混叠。
- 名义分段点向前预留搜索半径，使搜索窗口在硬上限内；分段不重叠，测试逐样本验证收到的信号守恒。
- 最新 MCP 为 2026-07-28，已按官方资料核对，同时实现任务单要求的旧版初始化。仅 JSON 响应，不提供可选 SSE / 会话功能。按任务单限制，MCP 忙时同时返回 HTTP 429 + Retry-After 与 `isError:true` 的 JSON-RPC 工具结果；直接 HTTP 客户端需保留错误正文。
- 白名单浏览器 OPTIONS 预检免 Bearer，只返回许可头，不调用后端；业务操作仍需 Bearer。未知 Origin / 无 Origin 的预检拒绝。
- notify 返回 queued 表示已接受后台处理，不能保证随后合成/播放成功；关闭时取消。旧版取消通知不保证中断宿主自己的设备线程。
- **待确认（任务单后续范围）**：VoiceMode 自定义地址配置、Claude Code / Codex 实际 MCP 接入、SenseVoice 服务用途许可、Edge 非官方接入政策、宿主真实适配器。
- **未测试**：真实模型/中文识别质量、实际设备与 VAD、真实网络 TTS、Linux/macOS 或其他架构、真实 IPv6 连接、大规模压力与长时间运行、所有损坏/罕见编码输入及模糊测试。超过 10 分钟的输入测试为 WAV；压缩格式的时长上限共用流式累计校验，但未构造十分钟压缩样本逐个测试。
- 所有自动测试均无真实设备、外网或 ffmpeg 运行依赖。ffmpeg 只用于开发期生成内嵌样本和人工命令验收，Python/ffmpeg 不属于 Rust crate 依赖。

## 最终自动验证原始输出

### cargo test --manifest-path libs/voice-service/Cargo.toml

```text
Compiling xiaoban-voice-service v0.1.0 (C:\Users\sdjz0\Documents\Claude\XiangBan\libs\voice-service)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 5.30s
     Running unittests src\lib.rs (libs\voice-service\target\debug\deps\xiaoban_voice_service-c43d6d0a1f1c8830.exe)

running 1 test
test security::tests::unanswered_consent_is_denied_after_thirty_seconds ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src\bin\xiaoban-voice-mcp-stdio.rs (libs\voice-service\target\debug\deps\xiaoban_voice_mcp_stdio-90bb2b2b53302db7.exe)

running 1 test
test tests::proxy_rejects_remote_tls_credentials_and_paths ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\fixtures.rs (libs\voice-service\target\debug\deps\fixtures-53ec8b9891d9367d.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\logging.rs (libs\voice-service\target\debug\deps\logging-63b28fdd44114440.exe)

running 1 test
test request_logs_exclude_tokens_text_headers_bodies_and_raw_paths ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests\mcp.rs (libs\voice-service\target\debug\deps\mcp-d7dfe44afe630d4e.exe)

running 8 tests
test notify_returns_before_playback_and_shutdown_cancels_background_work ... ok
test modern_discovery_metadata_headers_and_results ... ok
test legacy_initialize_notification_ping_list_and_all_tools ... ok
test absent_devices_are_hidden_and_uncallable ... ok
test tool_deadline_is_bounded_and_releases_speaker ... ok
test listen_empty_silent_busy_and_concurrent_limit ... ok
test stdio_binary_round_trip_legacy_and_modern_without_stdout_noise ... ok
test invalid_protocol_parameters_and_mcp_security ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests\openai.rs (libs\voice-service\target\debug\deps\openai-aec0d457897d88d5.exe)

running 11 tests
test rejects_bad_start_config ... ok
test limits_json_and_chunked_bodies ... ok
test authentication_health_host_origin_and_preflight ... ok
test third_concurrent_transcription_is_busy_and_slots_recover ... ok
test consent_denial_retry_cache_and_identity_capability_separation ... ok
test resampling_preserves_sine_frequency_duration_and_averages_stereo ... ok
test all_backend_error_variants_have_openai_envelopes ... ok
test speech_voice_mapping_validation_mp3_and_wav ... ok
test transcriptions_decode_all_formats_and_return_text_or_json ... ok
test low_energy_segmentation_is_lossless_bounded_and_chinese_joined ... ok
test transcription_validation ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s

   Doc-tests xiaoban_voice_service

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### cargo clippy --manifest-path libs/voice-service/Cargo.toml --all-targets -- -D warnings

```text
Checking xiaoban-voice-service v0.1.0 (C:\Users\sdjz0\Documents\Claude\XiangBan\libs\voice-service)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.29s
```

### cargo check --manifest-path libs/voice-service/Cargo.toml --no-default-features

```text
Checking tokio v1.54.1
    Checking hyper v1.12.0
    Checking tower v0.5.3
    Checking hyper-util v0.1.21
    Checking axum v0.8.9
    Checking xiaoban-voice-service v0.1.0 (C:\Users\sdjz0\Documents\Claude\XiangBan\libs\voice-service)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.86s
```

