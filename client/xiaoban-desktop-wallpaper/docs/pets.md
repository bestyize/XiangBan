# 桌面宠物主题

## 图片与资源包

支持植物、动物、玩偶与各类自创宠物形象。向日葵、郁金香、石莲、玉露、熊童子使用透明 WebP 写实素材；原绿叶和白色小雏菊继续复用原皮肤。五款的主题和植物 id 不变，已有实例直接换上新素材，位置、大小和心情不迁移。素材位于 `src/pets/assets/`，花盆和表情仍由原角色引擎绘制。

桌宠已接入通用资源包，`manifest.json` 为 `{"schemaVersion":2,"kind":"pet"}`，另有 `pet.json` 和 `assets/` 素材。桌宠页通过「导入宠物」导入标准 ZIP 资源包；自定义主题提供 ZIP 导出及论坛分享。导入只保存作品，不自动添加桌面实例；同 id 拒绝覆盖，官方主题不能覆盖。资源包不含实例、桌面位置、聊天或账号信息。

`pet.json` 沿用下文结构，包含图片素材的主题 `schemaVersion` 必须为 2。部件 `shapes` 新增：

```json
{"type":"image","src":"assets/flower.webp","x":96,"y":-29,"width":128,"height":185,"opacity":1}
```

图片与矢量部件可以混合，复用 `pivot` 与 `motion`。宠物设计坐标平移 y=-100，根部约 `(160,156)`；图片的 y 允许 -64..320，x 为 0..320，宽高 1..320。图片透明处按 alpha 命中测试，不占用整块图片矩形。图片可为透明 PNG、WebP 或 GIF；动态 WebP/GIF 保留原始字节。

ZIP 中 `src` 使用 `assets/` 相对路径，最多四层，只能包含字母、数字、点、下划线、连字符，不允许隐藏文件、上级目录或外链。每张最多 4 MiB，每主题最多 60 个图片图形、素材合计最多 16 MiB，整个 ZIP 及解压后不超过 32 MiB。缺图、未引用素材、扩展名与文件头不匹配都会在保存前拒绝。

运行和持久化时图片解析为 `data:image/png|webp|gif;base64,...`，因此单份数据可以离线显示，无须额外文件权限。ZIP 导出时重新提取图片，使用内容哈希文件名并去重。

AI 先读 `pet_read_format` 和 `pet_catalog`。读取完整主题时，图片 `src` 会返回 `asset:<内容哈希>`，避免把二进制编码塞进对话。调整部件尺寸、锚点、动作或另存主题时保留这些引用，`pet_theme_save` 会从本机已存在的主题恢复素材；不能编造引用。新增写实素材先导入 ZIP，不要让语言模型编造 base64。新建普通矢量形象仍可直接使用第一版格式。保存作品后需 `pet_add` 添加实例。

保留小伴原来的圆肚白瓷花盆、材质、屏幕、25 种表情、眨眼、睡眠和戳戳互动，开放宠物形象与部件创作。官方主题和自定义主题并列，主题内可包含多款宠物；同款可添加多只，每只是独立实例。

## 使用

在桌面宠物页选择主题并添加；在“桌面上的伙伴”中管理每只，或右键桌面上的宠物打开管理抽屉。“宠物形象”可直接更换，名称、位置、大小、心情和开关保留。每只支持拖动、独立显示、贴桌面或始终置顶；全局显示、暂停、帧率和桌面互动联动仍适用。桌面实例不设数量上限。

“AI 创作宠物”打开聊天草稿。AI 先调用 `pet_read_format` 和 `pet_catalog`，按声明式格式创作、通过 `pet_theme_save` 保存，再调用 `pet_add` 放置。通过 `pet_update` 的 `themeId` / `petId` 可为现有宠物更换形象。作品的保存与删除遵循现有聊天权限规则，调整应用内实例不额外请求确认。

## 主题 JSON（schemaVersion=1）

```json
{
  "schemaVersion": 1,
  "id": "my-garden",
  "name": "我的小花园",
  "description": "给熟悉的小伴换一株植物。",
  "color": "#91af6a",
  "pets": [{
    "id": "little-leaf",
    "name": "小叶芽",
    "description": "安静摇摆的一片新叶。",
    "kind": "plant",
    "parts": [{
      "id": "sprout",
      "name": "嫩芽",
      "pivot": [160, 157],
      "motion": "sway",
      "shapes": [
        {"type":"path","d":"M160 157L160 93","fill":"none","stroke":"#6b9653","strokeWidth":7},
        {"type":"path","d":"M160 108Q106 104 99 58Q151 57 160 108Z","fill":"#91af6a"}
      ]
    }]
  }]
}
```

植物在统一 320×320 坐标中设计，宿主将它平移 y=-100 放入原花盆；根部接到 `(160,157)`，建议图形位于 x=64..256、y=24..160，考虑摇摆和身体动作，顶部及左右留空。不要绘制花盆、脸、背景或投影。`motion` 可为 `none / sway / left / right`，后两种接原表情耳朵角度。各部件独立设置旋转中心。

图形只支持 `path(d)`、`ellipse(x,y,rx,ry)` 和 `rect(x,y,width,height,rx)`；颜色为六位 hex 或 `none`，支持 `stroke / strokeWidth / opacity`。包不支持 HTML、JavaScript、外链和任意 SVG 标签。`builtinPlant` 可选 `pot` 或 `pot-white-flower` 来复用原素材，此时 `parts` 必须为空；自创植物省略该字段。

主题和植物 id 使用小写字母、数字、`-`、`_`，最长 48 字符；名称最多 24 个 UTF-16 单位，描述最多 160；单文件 512 KiB，每主题最多 40 株，每株最多 16 部件、每部件 100 图形。这些是作品文件的边界，和桌面实例数量无关。

## 保存与兼容

实例保存在 `config/pets.json`，自定义主题保存在 `pets/themes/<id>.json`。首次没有实例文件时迁移旧盆栽，保留旧植物、尺寸、心情、显示与层级偏好；旧版没有持久化坐标，迁移到默认桌面位置。原偏好仍保留；随后单盆修改只写实例文件。

官方主题不能覆盖或删除。修改自定义主题需要显式 `overwrite=true`，并保留仍有桌面实例引用的植物；删除主题前需移除其所有桌面实例。文件损坏或读写失败会报错并保留原文件。每个原生宠物窗口只能获取、命中测试、拖动和管理自己，无法修改其他实例或写入作品。

## 定向验证

```sh
npx vitest run src/pets src/ui/pages/companion-page.test.ts src/companion src/ui/control-store.test.ts src/ui/ControlShell.test.ts
npx vue-tsc --noEmit
cargo test --locked --manifest-path src-tauri/Cargo.toml pet
cargo test --locked --manifest-path src-tauri/Cargo.toml the_assistant_has_every_tool_it_needs_and_no_others
```

自动检查覆盖原两款设计的逐表情复用、多盆持久化、同株重复添加、单盆换株、作品保护、包校验、原生命令边界与 AI 格式。Windows 桌面层级、跨屏拖动以及多窗口性能仍需实机验收。

