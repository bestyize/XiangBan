# xiaoban-infer：小伴桌面的本地推理引擎

小伴桌面需要本地推理时，从魔搭仓库 `bestyize/xiaoban-asr` 下载本程序和所需模型，放在应用数据目录的 `inference/` 下。程序通过标准输入输出与小伴桌面通信，每行一个 JSON。

内含 sherpa-onnx（静态链接其自带的 onnxruntime 1.28）。不直接经 sherpa-onnx 的能力用 `ort` 的 alternative-backend 模式复用同一份 onnxruntime，不另带运行库。

一个进程只提供一种能力，启动时指定：`xiaoban-infer --capability <能力> --dir <模型目录>`。不同能力分开运行，互不阻塞（例如识别期间不影响逐帧的语音检测）。

| 能力 | 作用 | 用到的模型 |
| --- | --- | --- |
| `asr` | 语音识别（SenseVoice） | `model.int8.onnx`、`tokens.txt` |
| `turn` | 判停：Silero VAD 判断有没有人说话，Smart Turn v3 判断这句话是否说完 | `silero_vad.onnx`、`smart-turn-v3.2.onnx` |

新增能力：在 `src/` 加一个带 `load` 和 `handle` 的模块，在 `src/main.rs` 的 `start` 里登记，请求类型加到 `src/protocol.rs`；小伴桌面侧用 `inference::Engine::start(目录, "能力名")` 启动。

## 协议

启动后先输出一行就绪：`{"status":"ready","version":1,"capabilities":["turn"]}`，列出本进程提供的能力。模型加载失败时输出 `{"status":"error","error":"…"}` 后退出。

| 能力 | 请求 | 响应 |
| --- | --- | --- |
| 全部 | `{"type":"ping"}` / `{"type":"exit"}` | `{"status":"pong"}` / 退出 |
| `asr` | `{"type":"transcribe","samples":[f32…]}`，0.1–20 秒 16 kHz | `{"status":"ok","text":"…"}` |
| `turn` | `{"type":"vad","stream":"id","pcm":[i16…]}` | `{"status":"probabilities","values":[…]}`，每 512 个采样一个 |
| `turn` | `{"type":"vadReset","stream":"id"}` | `{"status":"pong"}` |
| `turn` | `{"type":"turn","pcm":[i16…]}`，0.1–20 秒 | `{"status":"probability","value":0.0–1.0}` |

出错（包括请求了本进程不提供的能力）时返回 `{"status":"error","error":"…"}`。

## 构建与发布

在 GitHub Actions 手动运行 **BuildXiaoBanInfer**，下载产物 `xiaoban-infer-macos-arm64` 和 `xiaoban-infer.exe`（附带 SHA-256）。本地构建：

```sh
cd tools/xiaoban-infer
cargo test --release && cargo build --release
```

构建时 sherpa-onnx 从 GitHub Releases 下载预编译库；网络不通时，先下载对应压缩包，再把 `SHERPA_ONNX_ARCHIVE_DIR` 指向它所在的目录。

上传到魔搭 `bestyize/xiaoban-asr` 仓库根目录的文件：

| 文件 | 来源 | 大小（字节） | SHA-256 |
| --- | --- | --- | --- |
| `xiaoban-infer-macos-arm64` | 上面的构建产物 | 见构建输出 | 见构建输出 |
| `xiaoban-infer.exe` | 上面的构建产物 | 见构建输出 | 见构建输出 |
| `model.int8.onnx`、`tokens.txt` | 魔搭 `pengzhendong/sherpa-onnx-sense-voice-zh-en-ja-ko-yue`（版本 `73eca476`） | 239233841、315894 | 见 `src-tauri/src/asr/engine.rs` |
| `silero_vad.onnx` | pipecat-ai 1.13.0 发布包 `pipecat/audio/vad/data/silero_vad.onnx` | 2327524 | `1a153a22f4509e292a94e67d6f9b85e8deb25b4988682b7e174c65279d8788e3` |
| `smart-turn-v3.2.onnx` | pipecat-ai 1.13.0 发布包 `pipecat/audio/turn/smart_turn/data/smart-turn-v3.2-cpu.onnx`（改名） | 8679182 | `2bb026316b14a660486a75b1733cd3fbab8c2fd0314dc9af7be49f8cca967e4f` |

模型的大小与 SHA-256 写死在 `client/xiaoban-desktop-wallpaper/src-tauri/src/asr/engine.rs` 和 `src-tauri/src/voice_turn/mod.rs`，下载后逐一校验。引擎文件可同时放一份到备用源 `https://wallpaper.xiangban.xyz/app-packages/`。模型许可见 [MODELS.md](MODELS.md)。
