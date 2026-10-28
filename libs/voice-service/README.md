# xiaoban-voice-service

独立 Rust 2021 库：把宿主注入的识别、MP3 合成、麦克风、扬声器能力开放为回环 HTTP 服务。不依赖 Tauri、GTK、系统音频设备或真实语音模型；不在库内访问文件或建立出站连接。OpenAI 接口和第二阶段 MCP 均已实现。

## 运行和接入

在本目录执行（本次验证使用 Rust 1.94.1 / Windows x64）：

```powershell
cargo test
cargo clippy --all-targets -- -D warnings
$env:XIAOBAN_VOICE_TOKEN = 'replace-with-a-random-secret-of-24-or-more-characters'
$env:XIAOBAN_VOICE_BIND = '127.0.0.1:58139' # 可省略，默认随机端口
cargo run --example serve_mock
```

示例自动允许所有同意请求，仅限开发验证。识别固定返回 `你好，小伴`；合成返回自生成的 440 Hz 短音 MP3，**不是实际朗读**。默认没有麦克风和扬声器，MCP 工具列表为空。

应用依赖可以关闭仅供 stdio 二进制使用的 HTTP 客户端：

```toml
[dependencies]
xiaoban-voice-service = { path = "../voice-service", default-features = false }
```

宿主实现 `backends.rs` 中的 async trait，把实例放进 `Arc` 后传给 `start`：

```rust,no_run
use std::sync::Arc;
use xiaoban_voice_service::*;

async fn serve(
    transcriber: Arc<dyn Transcriber>, synthesizer: Arc<dyn Synthesizer>,
    consent: Arc<dyn Consent>, token: String,
) -> std::io::Result<RunningServer> {
    start(ServerConfig { token, ..ServerConfig::default() }, Backends {
        transcriber, synthesizer, microphone: None, speaker: None, consent,
    }).await
}
```

- `Transcriber` 接受 16 kHz、单声道、有限且在 `-1..=1` 内的 `Pcm`；覆盖 `max_samples()` 声明单段上限（默认 320000）。库负责分段、并发控制和合并，不负责模型加载。
- `Synthesizer::voices()` 默认音色放第一项；`synthesize()` 必须返回非空 MP3，MIME 为 `audio/mpeg`，不能在这里自动播放。`voice` 已经解析为实际后端 ID，`speed` 可为 `None`。
- `Microphone` 自行处理录音、说话结束检测和真实设备占用，必须遵守 `max_secs`，没说话可返回空 `Pcm`。`Speaker::play()` 播完才返回。
- `Consent` 连接宿主确认 UI，明确展示客户端标签和能力；不要把示例的无条件允许搬进产品。询问未完成 30 秒即拒绝。
- 后端必须异步、可取消，不能在 async 方法里执行阻塞推理或不可中断的设备操作；重型工作由宿主管理。库无法终止宿主自行派生的线程或网络请求。
- `RunningServer.addr` 给出实际端口；`shutdown().await` 停止接收、取消后台提示并等待在途请求结束；Drop 也会发送关闭信号。

## HTTP 接口

所有业务请求携带 `Authorization: Bearer <token>`；客户端 `base_url` 设为 `http://127.0.0.1:<port>/v1`。

| 接口 | 输入 / 输出 |
| --- | --- |
| `GET /health` | 无需令牌，只有 `{"ok":true}`；仍校验 Host / Origin |
| `POST /v1/audio/transcriptions` | multipart：必填 `file`、`model`（任意字符串，忽略）；可选 `language`、`prompt`（忽略）、`response_format=json\|text` |
| `POST /v1/audio/speech` | JSON：`model`（任意 JSON 值，忽略）、`input`、可选 `voice`、`speed`、`response_format=mp3\|wav` |
| `GET /v1/audio/voices` | `{"voices":[{"id","name","locale","gender"}]}`；使用 Synthesize 同意 |
| `POST /mcp` | JSON-RPC 2.0；同时接受新版和旧版 MCP，见下文 |
| `GET /mcp` | 405，不提供独立 SSE 流 |

转写按文件内容识别 WAV PCM16/24/32、WAV float32、MP3、FLAC、OGG Vorbis，不依赖上传的扩展名。多声道平均，流式重采样至 16 kHz：源采样率高于 16 kHz 时先做 Blackman 窗 sinc 低通（截止 7 kHz、过渡带 2 kHz）再取样，9 kHz 以上的成分衰减 80 dB 以上、不会折叠进语音频段，通带内 0.3–6 kHz 幅度误差在 2% 内，不引入时间偏移；不高于 16 kHz 时用线性插值；有效采样率 1..384000 Hz，1..8 声道。WebM / Opus / M4A 返回 415，并列出支持格式。损坏的已支持格式返回 400。空音频或不足 0.1 秒返回 `audio_too_short`；解码时即限制总时长 10 分钟，超过返回 413。

每段不超过后端上限。为让 ±1.5 秒搜索窗口完全落在硬上限内，名义切点设置为 `max_samples - radius`，`radius=min(1.5秒,max_samples/4)`；搜索最低局部均方能量（约 20 ms 窗口），不重叠、不丢采样。`language=zh*` 无分隔拼接，其他情况用空格。`verbose_json` / `srt` / `vtt` 等返回 400 `unsupported_response_format`。

朗读文本按 Unicode scalar 计数，去空白后不得为空，最多 4000 字符；语速范围 0.25..=4.0。未指定音色用第一项；alloy / ash / ballad / coral / echo / fable / nova / onyx / sage / shimmer / verse / marin / cedar 映射默认音色。其他未知音色返回 `unknown_voice`，消息最多列出 10 个可用 ID。MP3 原样返回，WAV 通过 MP3 解码后封装为单声道 16 kHz PCM16 WAV。

HTTP 错误统一为 `{"error":{"message":"...","type":"invalid_request_error","code":"..."}}`；不回显后端错误字符串。

| 情况 / VoiceError | HTTP / code |
| --- | --- |
| 参数、JSON、multipart 错误 | 400 / 对应参数码（如 `invalid_json`） |
| 缺失或错误令牌 | 401 / `unauthorized`，相同消息 |
| Host / Origin / 用户拒绝 | 403 / `forbidden_host`、`forbidden_origin`、`consent_denied` |
| 请求体超过上限 | 413 / `body_too_large` |
| TooLong | 413 / `audio_too_long` |
| Unsupported | 415 / `unsupported_audio_format` |
| Busy / 两个识别槽均占用 | 429 / `busy`，`Retry-After: 2` |
| Unavailable | 503 / `unavailable` |
| InvalidInput | 400 / `invalid_input` |
| Cancelled | 408 / `cancelled` |
| Internal | 500 / `internal_error` |
| 后端超时 | 504 / `timeout` |

## MCP 与 stdio

2026-10-28 已查阅官方 [最新版本入口](https://modelcontextprotocol.io/specification/latest)，当日指向 **2026-07-28**。该版逐请求携带元数据；任务单中的初始化属于旧版。因此提供双版本模式：

- **新版 2026-07-28**：实现 `server/discover`、`tools/list`、`tools/call`、`ping`；检查 `params._meta` 的 `io.modelcontextprotocol/protocolVersion`、`clientInfo`、`clientCapabilities`，以及对应 `MCP-Protocol-Version`、`Mcp-Method`、`Mcp-Name` 头；支持名字的 Base64 sentinel 编码。缺失/不匹配头返回 400 / `-32020`，未知版本返回 400 / `-32022` 并列出支持版本，未知方法返回 404 / `-32601`；成功带 `resultType: complete`。
- **旧版 2025-11-25、2025-06-18、2025-03-26**：实现 `initialize` → `notifications/initialized` → 工具调用，以及 `ping`。初始化不支持的旧版本协商为 2025-11-25；后续头声明未知版本返回 400；未带版本头按 2025-03-26 处理。无需会话 ID，客户端仍应遵守初始化顺序。
- 每个 POST 一条消息，要求 `Accept: application/json, text/event-stream`，服务选择单个 JSON 响应；接受的通知返回无正文 202。不提供 SSE、会话、恢复、进度、服务端请求或订阅能力，也不声明这些能力。

协议来源：[新版 HTTP](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/streamable-http)、[新版版本机制](https://modelcontextprotocol.io/specification/2026-07-28/basic/versioning)、[发现](https://modelcontextprotocol.io/specification/2026-07-28/server/discover)、[旧版 HTTP](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports)、[旧版初始化](https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle)。

| 工具 | 参数 | 行为 / 同意 |
| --- | --- | --- |
| `speak` | `text`、`voice?` | 合成并播放完，返回 `{"spoken":true}`；Synthesize + Speak |
| `listen` | `max_seconds?`（默认 15，1..30 整数） | 开麦后识别，返回 `{"text":"..."}`，空/静音返回空串；Listen + Transcribe |
| `converse` | `text`、`voice?`、`max_seconds?` | 先朗读再收听；开始前批准四项能力 |
| `notify` | `text` | 后台合成/播放，返回 `{"queued":true}`；Synthesize + Speak |

结果同时提供 JSON 文本 content 与 structuredContent。麦克风缺失隐藏 listen / converse；扬声器缺失隐藏 speak / notify / converse；调用隐藏工具返回 JSON-RPC `-32602`。同一时刻只有一个 listen（包括 converse 的整个过程）及一个播放操作；识别槽与 HTTP 转写共用。MCP 的 Busy 是工具层错误：HTTP 200，结果为 `isError:true`，`structuredContent` 里带 `{"busy":true,"retry_after_seconds":2}`，收听忙提示“麦克风正被占用”，扬声器忙提示“扬声器正被占用”。不返回 429，免得 MCP 客户端把它当传输错误而看不到提示。鉴权/同意拒绝仍是 HTTP 401/403。

每个工具整体最多 120 秒（含同意），麦克风单次另有 `max_seconds+5` 秒截止。notify 仅表示已排入后台，失败不会异步通知客户端；持有播放槽防止无限堆积，服务关闭会取消后台工作。静音判定只做空音频/过短或全部样本振幅低于 0.0001 的保底，真实说话检测归麦克风实现。

只支持 stdio 的客户端可运行：

```powershell
$env:XIAOBAN_VOICE_URL = 'http://127.0.0.1:58139'
# 令牌同服务端；建议环境变量，避免秘密出现在进程参数中
$env:XIAOBAN_VOICE_TOKEN = 'replace-with-a-random-secret-of-24-or-more-characters'
cargo run --bin xiaoban-voice-mcp-stdio
# 也接受 --url URL --token TOKEN
```

这是仅面向**本服务** JSON 响应模式的薄代理，不是通用远程 MCP 客户端：只访问显式回环 HTTP 地址，localhost 规范化为 127.0.0.1，不读取代理、不跟随重定向、不启用 TLS；每行 JSON-RPC，stdout 只输出协议消息，最多 32 个在途请求、单行/响应最多 1 MiB、HTTP 最多 185 秒。新版按消息元数据补对应头，旧版记住初始化返回版本。stdin EOF 后排空在途请求退出；取消通知会中止代理等待，但不保证中断服务端正在运行的宿主设备操作。

## 安全模型

- 只允许 `127.0.0.0/8` 或 `::1` 绑定；Host 严格只认 `127.0.0.1`、`localhost`、`[::1]`（可带合法端口），防 DNS 重绑定。即使绑定 127.0.0.2，调用也应使用允许的 Host。
- token 必须是 24..4096 个可见 ASCII 字符，无默认秘密；宿主负责用密码学随机源产生、保存及轮换。比较 SHA-256 摘要的固定 32 字节，使用 subtle 常量时间比较，不泄露哪个部分错误。
- Origin 必须精确命中 `allowed_origins`，无 Origin 放行；从不发送通配 CORS。OPTIONS 默认拒绝；白名单 Origin 的预检只返回许可头、不执行能力。预检本身免 Bearer 以支持浏览器，后续业务请求仍需 Bearer。
- ClientInfo.name 优先 X-Client-Name，再 User-Agent，再 unknown；缓存键是完整 ClientInfo（含可选 User-Agent）+ Capability。只缓存允许，拒绝下次重问，缓存仅存内存。标签由持令牌的客户端自报，**不是独立的可信身份隔离**。
- 同意请求串行，排队和询问合计最多 30 秒；重启清空授权缓存。所有相关能力批准前不打开设备。
- 完整请求体（含 multipart 包装）默认最多 25 MiB；读取最多 30 秒，含 chunked 也受限制。完整请求截止 180 秒（`/v1/audio/transcriptions` 放宽到 660 秒，因为最长要处理 10 分钟的音频）；识别/合成单次最多 120 秒。最多 2 个识别，1 个 listen；抢不到立即失败。
- 解码在阻塞线程池，持有识别许可；异步请求被取消时不会提前释放正在解码的许可。解码输出最多 960 万 f32，约 38.4 MB，另有上传正文及编解码器开销。
- 库不落盘、不出站。stdio 代理仅对本服务发回环请求，是单独的二进制入口。宿主注入的合成/识别实现是否联网由宿主负责。
- 只产生固定路由标签、状态码、耗时、大小的 tracing 事件，不记录正文、原始路径/查询、身份头、token 或识别文本。宿主额外中间件/后端的日志政策由宿主负责。

## 依赖与许可

库选择 MIT / Apache-2.0 / BSD 分支；根据任务委托人的明确回复，额外允许 **Unicode-3.0 传递依赖**。未使用 MPL 的 Symphonia，未使用系统编解码器或音频驱动。

| 直接依赖 | 选择的许可 / 用途 |
| --- | --- |
| async-trait、serde、serde_json | MIT 或 Apache-2.0；trait / 数据格式 |
| axum、tokio、tracing | MIT；HTTP / 异步 / 脱敏事件 |
| hound、claxon | Apache-2.0；WAV / FLAC |
| lewton | MIT 或 Apache-2.0；OGG Vorbis |
| puremp3 | MIT（从 MIT OR CC0 中选 MIT）；纯 Rust MP3 |
| sha2 | MIT 或 Apache-2.0；令牌摘要 |
| subtle | BSD-3-Clause；常量时间比较 |
| base64 | MIT 或 Apache-2.0；MCP 编码和小样本 |
| reqwest（stdio 可选，测试也使用） | MIT 或 Apache-2.0；无默认功能、无 TLS 的回环客户端 |
| tracing-subscriber（仅测试） | MIT；收集日志断言 |

锁定版本及传递依赖见 Cargo.lock 和 [依赖许可快照](DEPENDENCIES.md)。Unicode 数据表由 unicode-ident、ICU 等传递引入，不含真实音频素材。测试使用的三个小压缩样本均由仓库里的 Python 源码生成并以 Base64 文本保存，最大 FLAC 约 11.6 KB；运行测试无需 ffmpeg、Python 或网络。

## 验证与已知限制

自动测试全部使用假后端及回环 TCP，`cargo test` 一条命令覆盖安全、OpenAI、格式/频率/时长、分段、同意超时、并发、错误映射、日志脱敏、新旧 MCP 和真实 stdio 子进程往返。具体命令、原始输出、curl 实测见 [验收记录](ACCEPTANCE.md)。

- 重采样：源采样率高于 16 kHz 时的低通是 FIR（48 kHz 源约 132 个抽头），长音频在调试构建下较慢（30 秒 48 kHz 约 1.3 秒），发布构建快得多；缓存只保留内核宽度加一个批量。7–8 kHz 之间属于过渡带，该频段的信息会被部分衰减，对语音识别影响可忽略。
- MP3 由 puremp3 解码；它不保证所有编码器/损坏文件的兼容性，不裁剪编码延迟/padding；总时长边界按解码得到的采样计算。音频整体驻留内存，HTTP 识别不流式返回。
- 没有真实设备、模型、Edge 网络服务、VoiceMode、Claude Code / Codex 配置端到端测试；仅 Windows x64 实测，未验证 Linux/macOS 编译。
- 旧版模式不建立会话，也不在服务器保存初始化状态；通知取消不会保证终止宿主自行启动的工作。无后台 notify 成功回执、进度及服务器推送。
- 独立 HTTP 请求和 MCP 工具均有截止；无法强制终止不配合取消的宿主推理线程。宿主应自己管理进程、设备占用和真正的 VAD。
- 待确认：VoiceMode 自定义地址的实际配置、真实设备中文对话、宿主适配器、Edge 非官方接入及 SenseVoice 对外提供服务的许可，均是任务单注明的后续工作。
