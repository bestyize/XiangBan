# xiaoban-community：社区内容包的客户端逻辑

把一个社区内容包安全地取到本机并看懂，不碰界面，也不决定怎么应用：应用必须由宿主（桌面端）在用户确认后进行。

| 模块 | 职责 |
| --- | --- |
| `link` | 解析 `xiaoban://community/apply/{kind}/{pack_id}?version=&sha256=` 与 `xiaoban://community/open/{target}/{id}`；严格校验，链接只能请求“展示什么” |
| `client` | 向论坛取下载信息（`GET /api/packs/{id}/download`），按链接里的 `sha256` 锁定版本；依次从主地址和官方镜像下载，**每个来源**都校验大小与 SHA-256，必须 HTTPS（本机回环仅用于测试） |
| `pack` | 本机检查 zip：条目数与解压总量上限、路径穿越、符号链接、`manifest.json` 与类型一致；纯数据类（effect、scheme、reminder、routine、task、goal）必须带 `payload.json` 且不能夹带其他文件；提醒与任务模板按类型限定规则种类 |

## 内容包格式

zip 根目录：

- `manifest.json`：`{ "schemaVersion": 1, "kind": "effect", "id": "aurora", "name": "极光", "version": "1.0.0" }`，与论坛、资源机的检查一致。
- `payload.json`（纯数据类必需）：`effect` 是桌面现有的特效包 JSON；`reminder`、`routine` 是 `{ "rules": [ … ] }`，规则字段沿用桌面提醒规则（`reminder` 只含 `custom`/`off-work`/`rest`，`routine` 例行事项只含按时只读执行的 `agent`）；`task` 任务模板是 `{ title, scenario?, prompt, requiredSkills?, expectedOutput?, risk? }`，只取这些字段并限制长度；`goal` 目标模板是 `{ outcome, verification?, constraints?, boundaries?, stopWhen? }`，**不携带验证命令和自动续跑**。
- 可选 `README.md`、`preview.png`。

压缩包大小上限与论坛一致：纯数据类与专家 ≤ 1 MB，组件、游戏、技能、工作流 ≤ 8 MB，壁纸 ≤ 32 MB。

## 验证

```bash
cd libs/community-packs
cargo test
```

测试覆盖：链接的各种畸形输入、版本锁定、主地址被篡改时回退镜像、截断与超长响应、HTTPS 限制、压缩包的各类不安全结构。
