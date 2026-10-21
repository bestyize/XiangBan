# agent-proactivity

产品无关的 **主动感知事件 → 策略判断 → 注意力预算 → 行动候选 → 反馈** Rust 核心库。

## 定位

- 与 `agent-kernel`、`agent-tools` **平级**，无需依赖它们、Tauri、操作系统 API、模型服务或网络。
- 宿主采集已授权的事件（如用户回到电脑、宠物触摸），只向本模块传入受限的事件类型及上下文。
- `Engine::observe` 只保存各事件类型的**聚合计数**，不保留事件原文、鼠标坐标、窗口名称、屏幕内容和用户身份。
- `suggest` 仅给出无副作用的行动候选；`Engine::consider` 对授权/免打扰/活动状态/去重/日预算做确定性检查并预约一次行动。
- 宿主决定是否发送气泡、启动 Agent、播放动作；模型不能自行批准权限，也不能覆盖全局预算。
- 反馈只接受实际发出的 action id，重复上报无效；连续三次明确关闭会触发冷却，忽略不视为负反馈。

## 边界

默认 `Policy::default().enabled == false`，未打开产品开关时不会授权任何主动行为；
`Context::default()` 同样缺少使用授权，不能误放行。
`Channel::Ambient` 不占中断次数，但仍受授权和免打扰约束。
自定义/自创桌宠不得通过资源包修改这些开关。

它不是 OS 行为监控器、Tauri 常驻服务、LLM 文案生成器、宠物人格/动画或 Agent 执行器；
未实现持续任务、后台模型调用、跨设备同步。事件的 `subject_id` 仅在内存中输入校验，
快照不保存它。话题键必须为不含个人文本的短 ID。每天按宿主提供的本地 `YYYYMMDD`
重置注意力额度；系统时间倒退时不恢复额度。

## 桌面接入

`client/xiaoban-desktop-wallpaper/src-tauri/src/proactivity.rs` 是宿主适配层：
复用 `presence_tick` 的首次活跃/归来/深夜事件，只保存聚合计数。
Stage II 进一步把宠物 hit-test 的悬停和点击接入本模块；**只有用户手动开启
「桌面宠物 → 桌面伙伴 → 主动陪伴」后**，宿主才可能调用 `consider` 授权展示
本地短句气泡，并通过 action id 记录反馈。未开启时不展示、不上报宠物点击聚合。
不新增操作系统轮询，不需要 LLM，不会保存屏幕内容或鼠标坐标。

注意力额度作用于当前新增的主动气泡；已有的休息提醒与欢迎表情尚未统一调度。
当前宿主没有 OS 级全屏识别，因此暂不能保证全屏时的静默；这是下一次整合的工作，
不能在当前文档里宣称已具备这一能力。

## 验证

```sh
cargo test --manifest-path libs/agent-proactivity/Cargo.toml
cargo check --locked --manifest-path client/xiaoban-desktop-wallpaper/src-tauri/Cargo.toml
```

测试覆盖默认禁用、授权、游戏与全屏、频控、跨日与时钟回拨、话题去重、反馈幂等、
忽略与关闭的差别、持久化恢复及悬停阈值。
