# 社区资源服务部署

社区作品包和精选播放器由资源机 `124.220.64.203` 的 `https://res.xiangban.xyz` 提供。论坛签发上传凭证、签名下载链接并校验回调；ZIP 内容包不是公开文件，仍需短期签名。封面使用公开的 `/img/{md5}.{ext}`，社区 ZIP 使用签名的 `/files/{md5}.zip`。公开官方安装包、模型和精选作品包也位于 `/files/`，不能把私有文件复制到公开目录。

- 服务：`xiangban-resource.service`，用户 `xb-resource`，监听 `127.0.0.1:55020`。
- 程序：`/srv/xiangban-resource/app`；数据：`/srv/xiangban-resource/data/`。
- 配置：`/etc/xiangban-resource/environment` 是 root 可读的 NUL 分隔 `KEY=value` 凭据，至少包括与论坛一致的 `RESOURCE_SECRET`。systemd 通过 `LoadCredential` 提供给 [`xiangban-resource-launch`](xiangban-resource-launch)；不要写入仓库或输出密钥。
- Nginx：TLS 站点包含 [`nginx-res.conf`](nginx-res.conf) 和 [`nginx-ideas.conf`](nginx-ideas.conf)。前者将上传和签名对象交给服务，后者提供沙箱播放器、展厅和内置游戏。论坛旧资源入口的代理模板见 [`nginx-files.conf`](nginx-files.conf)。
- 论坛图片和附件由 `xiangban-forum-images.service`（[`forum-image-cache.mjs`](forum-image-cache.mjs)，监听 `127.0.0.1:55021`）保存到 `data/forum-content/objects/<md5>.<ext>`。图片的公开访问在资源机逐次向论坛确认可见性；附件先在论坛校验阅读和购买权限，再签发短期下载链接，不能把这个目录设为公开静态目录。论坛仅允许资源机访问 [`nginx-forum-origin.conf`](nginx-forum-origin.conf) 的内部源站接口。
- 网页和手机版正文图片、附件先向论坛申请限时上传凭证，再直接上传到资源机；资源机核对大小和 SHA-256 并签发回执，由论坛校验回执后写入数据库。运营素材和旧上传接口由论坛转存到资源机。`data/forum-archive/` 是迁移时的私有快照，不是实时备份。
- 精选播放器、展厅和内置游戏分别位于 `/srv/xiangban-resource/idea-players-v1/`、`gallery-exhibits/`、`builtin-games/`；论坛静态网页资源位于 `/srv/xiangban-resource/forum-web/`，公开文件在 `/var/www/res.xiangban.xyz/`。具体更新方法见论坛 README。
- 论坛设置 `FORUM_RESOURCE_BASE_URL=https://res.xiangban.xyz`；资源服务回调 `https://chat.xiangban.xyz/api/internal/pack-uploaded`。

本地构建 Linux 产物后上传，再重启服务；不要在云服务器编译。验证 `/health`、封面、签名下载、未签名拒绝和实际上传回调，并用浏览器检查跨域作品。`services/server-resource/scripts/backup.sh` 备份社区服务的数据目录，包括论坛资源；公开文件、网页、播放器、展厅和内置游戏目录须另外做异地备份。配置和数据库备份需保存在受保护的位置。脚本需要另外配置备份目标，单机上的副本不能替代异地备份。

网站内置游戏为保留相对引用，以目录路径提供；更新后运行 `index-public-assets.py --source /srv/xiangban-resource/builtin-games --objects /srv/xiangban-resource/public-objects --manifest /srv/xiangban-resource/builtin-games-manifest.json --url-prefix https://res.xiangban.xyz/builtin-games/`，以 MD5 硬链接保存内容，并在清单中记录 SHA-256 与原路径。两目录须在同一文件系统上；同名对象会校验 SHA-256，冲突则停止。

论坛公开网页资源更新后，同样对 `forum-web/` 运行索引脚本；不要对包含私有附件的 `forum-content/` 建立公开索引。更换网页产物时先同步资源机，再更新论坛的 HTML 并重启论坛，保留旧的带哈希文件以供已打开的页面使用。
