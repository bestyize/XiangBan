# compose-typecheck

云端沙箱编不了 Compose（androidx 依赖在 Google Maven 上，访问不到），所以手机端界面（`client/xiaoban-multiplatform/business/community-native`）没法直接编译。这个目录用**仿制的 Compose / Material3 / Coil 接口**（`stubs/`）加上真实的 Kotlin 源码做一次类型检查，能抓出未定义的名字、参数名写错、类型不匹配、作用域用错这类错误。

```sh
tools/compose-typecheck/sync.sh        # 没有输出 = 通过
```

局限：仿制接口是照真实签名手写的，**不是真实的**。通过只说明“按这些签名写是对的”，真实编译（CI 的 `android` 任务）仍可能报错，尤其是 Compose 编译器自己的规则（在非 Composable 上下文里调用 Composable 等）。新用到某个 Compose API 时，先在 `stubs/` 里补上它的真实签名再检查。

第一次运行需要联网下载 Kotlin 插件和库（脚本用了 `--offline`，所以先手动运行一次 `gradle compileKotlin --no-daemon` 让依赖进缓存）。`build-src/` 是生成的，已在 `.gitignore` 里。
