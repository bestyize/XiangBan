# AI 图片生成上线记录

2026-11-01（北京时间）核实并配置。AI 网关与 New API 的实际生产部署都在香港服务器，历史设计里的首尔同机部署设想不代表当前部署位置。服务器与密钥清单仍按本机 SSH 运维记录核实；不要为连接旧地址而忽略 SSH 主机指纹校验。

## 线上配置

- AI 网关：`server-ai.service`，目录 `/opt/server-ai`，监听 `127.0.0.1:55012`，对外地址 `https://ai.xiangban.xyz`。
- 上游：现有 `new-api.service`，监听 `127.0.0.1:3000`；复用现有 Vertex AI 渠道（ID 2、`default` 分组），不新增或下发上游密钥。
- 在该渠道和 `abilities` 登记 `gemini-3.1-flash-image`，无需模型别名映射；现有 Vertex 默认区域为 `global`。
- New API 的 `ModelPrice` 为此模型单独增加 `0.0672` 美元/次；每个原生请求生成一张 1K 图片，对应本站 `33,600` 点。该数值采用 [Google 公布的 1K 图像输出参考价格](https://ai.google.dev/gemini-api/docs/pricing#gemini-3.1-flash-image)，本站当前按请求定价，不单列提示词 token 费用。其余模型价格不变。
- `product_media`：`product_id=wallpaper`、`capability=image`、`model=gemini-3.1-flash-image`、`enabled=true`、`requires_paid=false`、`sizes=["1024x1024"]`。沿用已有试用/付费余额检查与对账机制；桌面和 Android 关联同一产品，但客户端仍需各自实现生图入口。

桌面对外仍调用 `POST /v1/images/generations`，不传模型名。网关将受支持的 Gemini 原生生图请求转换为 New API 的 `generateContent`，通过当前调用者自己的 New API Key 扣费。其他生图模型继续走原来的 Images 接口。多张请求按张顺序执行，无自动重试；后续失败时返回已经生成的图片。

## 验证

- 本地 `media::unit` 六项测试通过，包含 Gemini 请求转换、尺寸校验、仅返回最终图片，以及排除思考内容、文本、模型名与用量。
- 产物在本机以 `cargo zigbuild --offline --locked --release --target x86_64-unknown-linux-gnu.2.35 --manifest-path services/server-ai/Cargo.toml` 构建，再上传；未在云服务器编译。
- 通过真实桌面账号令牌调用线上 `/api/ai/tiers`，确认 `media.image.available=true`、尺寸为 `1024x1024`。
- 调用统一生图接口，真实返回一张 1024×1024 PNG（141,070 字节，约 12 秒）；响应仅含 `created`、`data` 和 `b64_json`，没有模型名或用量。
- New API 消费记录 ID `6396`：Vertex 渠道、输入 16 token、输出 1120 token、扣除 33,600 点。已确认既有对账将客户端余额从 3,430,797 更新为 3,397,197，差额正好为 33,600 点；额度展示仍可能延迟约 60 秒。

## 备份与回滚

服务器备份目录：`/opt/server-ai/backups/image-enable-20261101-140312/`，仅服务器管理员可访问。

- `configuration.before.json`：变更前的渠道模型列表、此模型的能力登记、原 `ModelPrice` 设置及图片媒体配置；不包含上游密钥。
- `server-ai.before`：替换前的服务二进制。
- 本次媒体启用写入 `admin_audit`，操作来源标记为 `ops:ssh`，目标为 `wallpaper/image`。

回滚时先停用 `wallpaper/image`，再将旧二进制复制到临时路径并原子替换 `/opt/server-ai/server-ai`，重启 `server-ai.service` 并检查 `/api/health`。需要撤销上游登记时，仅移除本次新增的模型、对应能力行及 `ModelPrice` 的该模型键；不要用整份旧价格 JSON 覆盖部署后其他人的配置。New API 当前每 60 秒同步配置，无需为价格修改重启全站模型网关。

不要删除用户额度桶、修改既有流水或手工覆盖 New API 钱包来回滚功能。
