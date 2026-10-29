# 本机运行

已配置独立的 PostgreSQL 17.11，监听 `127.0.0.1:5432`。
数据库为 `wallpaper_collect`，应用账号为 `wallpaper_app`，随机生成的连接密码保存在已被 Git 忽略的 `.env` 中。

在项目目录使用 PowerShell 启动数据库和网站：

```powershell
.\scripts\start-local.ps1
```

网站：http://localhost:8080/

管理后台：http://localhost:8080/admin

初始后台账号：`admin` / `123456`。可在后台「账户」中修改。

数据库运行文件、持久化数据、日志和数据库管理凭据位于：

```text
%LOCALAPPDATA%\WallpaperCollect\postgresql\
  runtime\          PostgreSQL 程序
  data\             数据库数据
  postgresql.log    数据库日志
  credentials.json  本机数据库管理凭据
```

网站日志位于 `target/wallpaper-server.log` 和 `target/wallpaper-server-error.log`。
关闭预览不会停止服务；重启电脑后可重新执行启动脚本。数据目录独立于代码和 `target` 构建目录。
