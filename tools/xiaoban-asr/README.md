# xiaoban-asr：小伴桌面的本地语音运行时

小伴桌面开启语音识别时，从魔搭仓库 `bestyize/xiaoban-asr` 下载本程序和模型，放在应用数据目录的 `asr/` 下。程序通过标准输入输出与小伴桌面通信，每行一个 JSON。

内含 sherpa-onnx（静态链接其自带的 onnxruntime 1.28），以两种模式运行：

| 启动方式 | 作用 | 用到的模型 |
| --- | --- | --- |
| `xiaoban-asr --dir <目录>` | 语音识别（SenseVoice） | `model.int8.onnx`、`tokens.txt` |
| `xiaoban-asr --turn --dir <目录>` | 判停：Silero VAD 判断有没有人说话，Smart Turn v3 判断这句话是否说完 | `silero_vad.onnx`、`smart-turn-v3.2.onnx` |

判停用 `ort` 的 alternative-backend 模式，复用 sherpa-onnx 已经链接进来的那份 onnxruntime，不另带运行库。两种模式分成两个进程，识别期间不会阻塞逐帧的语音检测。

## 协议

启动后先输出一行就绪：`{"status":"ready","version":2,"features":["transcribe"]}`（判停模式为 `["turn"]`）。第一版运行时只输出 `{"status":"ready"}`，小伴桌面据此判断它不支持判停。

| 请求 | 响应 |
| --- | --- |
| `{"type":"transcribe","samples":[f32…]}`，0.1–20 秒 16 kHz | `{"status":"ok","text":"…"}` |
| `{"type":"vad","stream":"id","pcm":[i16…]}` | `{"status":"probabilities","values":[…]}`，每 512 个采样一个 |
| `{"type":"vadReset","stream":"id"}` | `{"status":"pong"}` |
| `{"type":"turn","pcm":[i16…]}`，0.1–20 秒 | `{"status":"probability","value":0.0–1.0}` |
| `{"type":"ping"}` / `{"type":"exit"}` | `{"status":"pong"}` / 退出 |

出错时返回 `{"status":"error","error":"…"}`。

## 构建与发布

在 GitHub Actions 手动运行 **BuildXiaoBanAsr**，下载产物 `xiaoban-asr-v2-macos-arm64` 和 `xiaoban-asr-v2.exe`（附带 SHA-256）。本地构建：

```sh
cd tools/xiaoban-asr
cargo test --release && cargo build --release
```

构建时 sherpa-onnx 从 GitHub Releases 下载预编译库；网络不通时，先下载对应压缩包，再把 `SHERPA_ONNX_ARCHIVE_DIR` 指向它所在的目录。

上传到魔搭 `bestyize/xiaoban-asr` 仓库根目录的文件：

| 文件 | 来源 | 大小（字节） | SHA-256 |
| --- | --- | --- | --- |
| `xiaoban-asr-v2-macos-arm64` | 上面的构建产物 | 见构建输出 | 见构建输出 |
| `xiaoban-asr-v2.exe` | 上面的构建产物 | 见构建输出 | 见构建输出 |
| `silero_vad.onnx` | pipecat-ai 1.13.0 发布包 `pipecat/audio/vad/data/silero_vad.onnx` | 2327524 | `1a153a22f4509e292a94e67d6f9b85e8deb25b4988682b7e174c65279d8788e3` |
| `smart-turn-v3.2.onnx` | pipecat-ai 1.13.0 发布包 `pipecat/audio/turn/smart_turn/data/smart-turn-v3.2-cpu.onnx`（改名） | 8679182 | `2bb026316b14a660486a75b1733cd3fbab8c2fd0314dc9af7be49f8cca967e4f` |

两个模型的大小与 SHA-256 写死在 `client/xiaoban-desktop-wallpaper/src-tauri/src/asr/engine.rs`，下载后逐一校验。运行时文件可同时放一份到备用源 `https://wallpaper.xiangban.xyz/app-packages/`。

新版运行时改用了新文件名：已装过第一版的用户下次准备语音时会自动下载新版；新版下载失败时继续用第一版识别，只是没有智能判停（按音量判停）。模型许可见 [MODELS.md](MODELS.md)。
