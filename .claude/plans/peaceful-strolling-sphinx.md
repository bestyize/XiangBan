# 在相伴账号中心（server-account）管理后台支持配置与测试 SMTP 邮件服务

## Context
当前相伴统一账号中心（`account.xiangban.xyz`）支持邮箱与手机号验证码注册登录。但发信渠道仅硬编码了腾讯云 SES 与控制台日志两种模式，且所有配置依赖环境变量，无法在后台动态修改。由于国内云厂商邮件服务需要主体企业认证与复杂的工信部备案/模板审核，接入个人或企业常用的标准 SMTP（如 163 邮箱、QQ 邮箱、飞书/企业微信、Resend 等）是让真实用户能立即收到验证码邮件的最佳路径。本方案在账号中心管理后台增加 SMTP 服务的动态配置、加密存储与在线测试发信功能。

## 架构与改动点

### 1. 后端依赖与数据存储 (`services/server-account`)
- **依赖**：引入纯 Rust 异步邮件库 `lettre`（启用 `tokio1-rustls-tls`, `smtp-transport`, `builder`，无需 OpenSSL 本地链接）。
- **数据库**：在 `db.rs` 的初始化迁移中增加 `system_settings` 表：
  ```sql
  CREATE TABLE IF NOT EXISTS system_settings (
      key        TEXT PRIMARY KEY,
      value_enc  BYTEA NOT NULL,
      updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
  );
  ```
  SMTP 的密码和完整配置采用 `Keys.sub.encrypt("smtp_config", &json_bytes)` 加密存储，保证数据库泄露时不泄露发信凭据。
- **发送器升级 (`src/sender.rs`)**：
  - 支持 `Sender::Smtp(SmtpSender)`，构造基于 `AsyncSmtpTransport<Tokio1Executor>` 的发送器。
  - 支持 SSL/TLS（端口 465）和 STARTTLS（端口 587/25）两种常见连接协议。
  - 邮件正文美化：生成包含清晰 6 位验证码、失效时间与防钓鱼提示的 HTML 邮件与文本纯文本回退。
  - 动态切换支持：`SenderManager` 或基于 `Arc<RwLock<Sender>>` 的热重载机制，保存配置即时生效。

### 2. 管理后台接口 (`src/handlers/admin.rs` & `src/main.rs`)
- `GET /api/admin/sender`：获取当前发信渠道类型（`console` / `smtp` / `tencent`）与 SMTP 参数（密码脱敏返回 `******`，同时返回 `has_password: true`）。
- `PUT /api/admin/sender`：保存发信设置，加密存储至数据库并热更新运行期 Sender，写入 `admin_audit` 审计日志。
- `POST /api/admin/sender/test`：在线测试发信。接收目标邮箱与表单参数（可测试当前表单未保存的参数），立即发送一封真实测试邮件并返回成功或具体的 SMTP 报错详情。

### 3. 前端管理后台页面 (`services/server-account/web`)
- 新增 `src/views/SenderView.vue`：
  - 渠道切换：控制台日志（开发模式） / SMTP 邮件服务 / 腾讯云服务。
  - SMTP 快捷预设填充（163 邮箱、QQ 邮箱、企业微信、Gmail、自定义）。
  - 配置表单：Host、Port、加密模式、发件邮箱、发件人密码/授权码、发件人名称。
  - 「在线测试发信」功能：输入测试邮箱，调用测试接口，实时反馈发信成功或错误堆栈。
  - 「保存配置」功能。
- 在 `App.vue` 侧边栏导航增加「发信设置」菜单项。

### 4. 验证与部署流程
1. 本地单元测试与编译：
   - 验证 SMTP 消息构建与配置加密/解密。
   - `npm --prefix services/server-account/web run build` 本地构建前端 dist。
   - `cargo test --locked` 验证全部 30 个现有测试通过。
2. 同步与部署到香港云服务器：
   - 遵循 `cloud_deploy_local_build.md` 原则，前端在本地编译完毕后仅同步静态 dist 与 rust 增量源码。
   - 远端执行增量编译并重启 `server-account.service`。
   - 打开 `https://account.xiangban.xyz/admin/`，使用 admin 账号登录，配置实际 SMTP 并进行发信测试。
