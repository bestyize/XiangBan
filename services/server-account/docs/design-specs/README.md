# 相伴账号中心管理后台 · 设计资源索引

本目录存放 `services/server-account/web` 管理后台的交互设计与重设计原型，源文件随仓库维护，不参与生产构建。全端统一浏览入口见[小伴全端设计稿总导航](../../../../docs/design-preview/index.html)。

## 设计稿清单

| 设计稿 | 状态 | 说明 | 预览方式 |
| --- | --- | --- | --- |
| [账号中心管理后台极简重设计原型](prototypes/account-admin-redesign.html) | 已合入 | 对齐小伴暖白极简 + 墨黑中性基调 + 双星日落珊瑚徽标（支持浅/深色切换、`⌘K` 全局跳转与查用户、桌面/平板/手机三端预览），覆盖登录页、概览与验证码来源 IP 监控、用户检索与侧滑抽屉、接入应用管理、发信通道（SMTP/腾讯云/阿里云 PNVS）与注册开关、结构化时间线操作日志与统一模态弹窗（彻底淘汰原生 `alert`/`confirm`/`prompt`） | 直接打开 HTML 或运行 `python3 docs/design-preview/serve.py --open` |

## 常用直达预览链接（启动 `serve.py` 后）

- [概览（浅色）](http://127.0.0.1:1455/services/server-account/docs/design-specs/prototypes/account-admin-redesign.html?page=dashboard&theme=light)
- [用户管理 + 详情抽屉](http://127.0.0.1:1455/services/server-account/docs/design-specs/prototypes/account-admin-redesign.html?page=users&drawer=user)
- [接入应用](http://127.0.0.1:1455/services/server-account/docs/design-specs/prototypes/account-admin-redesign.html?page=clients)
- [发信与注册设置](http://127.0.0.1:1455/services/server-account/docs/design-specs/prototypes/account-admin-redesign.html?page=sender)
- [结构化操作日志](http://127.0.0.1:1455/services/server-account/docs/design-specs/prototypes/account-admin-redesign.html?page=audit)
- [极夜深色](http://127.0.0.1:1455/services/server-account/docs/design-specs/prototypes/account-admin-redesign.html?page=dashboard&theme=dark)
- [登录页](http://127.0.0.1:1455/services/server-account/docs/design-specs/prototypes/account-admin-redesign.html?view=login)
