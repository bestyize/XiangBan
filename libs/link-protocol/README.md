# xiaoban-link-protocol

小伴手机与桌面互联协议 v1 的**参考实现**。规范见 [设备互联协议](../../docs/agent/设备互联协议.md)。

只做密码学和协议规则，不含网络、存储、界面：

- `keys`：P-256 设备密钥与 ECDH。
- `pairing`：二维码载荷、手机的配对证明、配对密钥 PRK。
- `envelope`：加密信封（AES-256-GCM，每个方向、每个会话一把密钥，明文补齐到 256 字节整数倍）。
- `replay`：接收端重放防护（按启动计数判断会话新旧，需持久化）。
- `vectors`：测试向量的生成。

```sh
cargo test                          # 协议测试 + 校验 vectors/v1.json 与实现一致
cargo run --example gen_vectors     # 改了协议后重新生成向量
```

`vectors/v1.json` 是两端共同的验收标准：桌面（Rust，直接用本库）和手机（Kotlin，`link-core`）都必须算出相同的公钥、共享秘密、PRK、证明和信封。向量里的私钥只是测试值，切勿用于真实设备。
