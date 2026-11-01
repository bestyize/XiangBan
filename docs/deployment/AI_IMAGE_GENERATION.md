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

## 9Router 的 GPT / Gemini 生图通路

同日追加检查确认，现有两个账户渠道都能够生图；最初的失败不能据此判断整个渠道不可用。9Router `0.5.112` 的 Images 适配没有向 HTTP 请求或 Antigravity executor 传递账户已配置的代理，Codex 的共享令牌刷新也遗漏了代理。直连时分别出现 GPT 403 和 Gemini 地区限制错误。修复代理后，GPT 另有旧访问令牌失效问题；已通过现有刷新凭据成功刷新并保存，无需重新登录。

- `ag/gemini-3.1-flash-image`：经原有 Antigravity 账户与代理，HTTP 200，返回一张图片（463,593 字节，约 11.7 秒）。
- `cx/gpt-image-1.5`：经原有 Codex Plus 账户与代理，HTTP 200，返回一张图片（1,026,495 字节，约 16.7 秒）。
- 账户刷新后同时保存访问/刷新令牌、有效期及刷新时间，沿用现有凭据持久化函数。凭据不进入脚本、仓库或终端输出。
- 产品当前默认图片模型仍为前述 Vertex Gemini；上述测试验证的是 9Router 上游接口，没有给桌面增加模型选择项或自动重试链。

可重复应用的运维补丁为 [patch_9router_image_proxy.py](patch_9router_image_proxy.py)。它仅支持当前安装版本与已核对的编译产物结构，先做精确匹配和 `node --check`；默认只检查，显式 `--apply` 才备份并替换文件，重复运行会跳过已安装的补丁。应用后重启 `9router.service`。例如在服务器上：

```sh
python3 patch_9router_image_proxy.py /usr/lib/node_modules/9router
sudo python3 patch_9router_image_proxy.py /usr/lib/node_modules/9router --apply --backup-dir /opt/9router-backups/image-proxy-<本次时间>
sudo systemctl restart 9router
```

补丁直接修正已安装的 JavaScript 产物，未在云服务器执行构建。9Router 升级或重装会覆盖这些修改，届时应检查上游是否已修复，再按新版本复核；不可向未知版本强行应用。

## GPT Image 2.5 后台可选模型

2026-11-01 追加登记 `gpt-image-2.5`，供 AI 后台的图片模型下拉框选择；产品当前默认仍为 Vertex Gemini，没有切换用户的默认生图模型。

- New API 新增 `9Router Images` 渠道（ID 6，OpenAI 兼容类型 1），复用现有 9Router 地址与凭据，模型 `gpt-image-2.5` 映射为 `cx/gpt-image-2.5`。为该模型登记 `default`、`vip`、`svip` 三个分组的能力。原 9Router 聊天渠道（ID 4）的模型、映射和高级路由保持原配置。
- 按用户在本次配置中明确选择的方案，将此模型 `ModelPrice` 设为 `0.0672` 美元/张，即当前默认分组每张 `33,600` 点，与本站 Gemini 生图价格一致。这是本站固定价格，并非 OpenAI 官方单张价格。
- [OpenAI 官方价格](https://developers.openai.com/api/docs/models/gpt-image-2.5-sunburst)按 token 计算。现有 9Router Images 响应只返回图片，不转发 Responses 的 `tool_usage.image_gen`；聊天模型自己的 token 不能代替图像用量计费，因此本次使用明确的按张定价，不把缺失的图像用量记为免费。
- `gpt-image-2.5` 是本次实测被 9Router/Codex 接受的兼容名称；官方 Images API 当前列出的 2.5 名称为 `gpt-image-2.5-sunburst` 和 `gpt-image-2.5-flare`，详见 [官方接口](https://developers.openai.com/api/reference/resources/images/methods/generate)。当前登记不承诺裸别名绑定某个固定变体；需要锁定变体时应使用明确的变体名并单独验证。
- 直接经 9Router 的 `cx/gpt-image-2.5` 已实测 HTTP 200，返回一张图片（1,183,315 字节，约 18.6 秒）。
- 使用当前桌面账号绑定的独立 New API Key 调用 New API `/v1/images/generations`，指定 `gpt-image-2.5`，HTTP 200，约 19.1 秒返回一张 PNG（1,009,165 字节；请求尺寸 `1024x1024`，实际输出 `1254x1254`）。消费记录 ID `6420`、渠道 6、扣除 `33,600` 点。响应没有图像用量，本次按张价格正常结算。

原 9Router 渠道是高级自定义类型 58，其路由只声明聊天、Responses、Messages、Gemini 等路径，没有 Images 路径。仅在原渠道登记模型会被请求路径过滤器排除，返回 503 `No available channel`；这是本次另建图片兼容渠道的原因。渠道内存缓存按现有 60 秒周期同步数据库，价格列表可见不代表图片路由已经通过验证。

配置快照保存在 `/opt/server-ai/backups/gpt-image-25-20261101/configuration.before.json`，只包含模型、映射、能力、原价格键和产品媒体配置，不含密钥；`registration.json` 记录新增图片渠道 ID 与价格。变更写入 `admin_audit`，操作来源 `ops:ssh`，动作 `newapi.model.register`，最终目标 `gpt-image-2.5`。

撤销登记时先停用新增渠道 6，再移除该渠道的三组同名能力和渠道行，以及 `ModelPrice` 的 `gpt-image-2.5` 键；如渠道后来新增了其他模型，应只移除本次模型与能力，不整行删除。不要用完整旧渠道或旧价格覆盖之后新增的配置。产品默认媒体配置无需回滚。

## 备份与回滚

服务器备份目录：`/opt/server-ai/backups/image-enable-20261101-140312/`，仅服务器管理员可访问。

- `configuration.before.json`：变更前的渠道模型列表、此模型的能力登记、原 `ModelPrice` 设置及图片媒体配置；不包含上游密钥。
- `server-ai.before`：替换前的服务二进制。
- 本次媒体启用写入 `admin_audit`，操作来源标记为 `ops:ssh`，目标为 `wallpaper/image`。

回滚时先停用 `wallpaper/image`，再将旧二进制复制到临时路径并原子替换 `/opt/server-ai/server-ai`，重启 `server-ai.service` 并检查 `/api/health`。需要撤销上游登记时，仅移除本次新增的模型、对应能力行及 `ModelPrice` 的该模型键；不要用整份旧价格 JSON 覆盖部署后其他人的配置。New API 当前每 60 秒同步配置，无需为价格修改重启全站模型网关。

不要删除用户额度桶、修改既有流水或手工覆盖 New API 钱包来回滚功能。

9Router 两个文件的原始备份分别为：

- `/opt/9router-backups/image-proxy-20261101/images-generations-route.js.before`，对应安装目录下 `app/.next-cli-build/server/app/api/v1/images/generations/route.js`。
- `/opt/9router-backups/image-proxy-refresh-20261101/oauth-refresh.js.before`，对应安装目录下 `app/.next-cli-build/server/chunks/3943.js`。

需要回滚补丁时，仅恢复上述两个产物文件并重启 `9router.service`，保留已经正常刷新的最新凭据。不要恢复中间调试版本或用旧数据库覆盖现有账户。
