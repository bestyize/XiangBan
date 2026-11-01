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
