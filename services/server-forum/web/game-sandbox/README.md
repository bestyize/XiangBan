# 网页游戏隔离宿主

该目录是独立的静态站点，不能放在论坛源或接收账号 cookie 的域名上。生产部署使用独立、无账号 cookie 的 HTTPS 站点（建议独立注册域名），设置论坛前端构建变量 `VITE_GAME_SANDBOX_URL=https://<独立域名>/`。该值不能与论坛同源。资源下载若跨源，资源服务器需要对论坛源允许无凭据 GET 的 CORS；本地构建后部署，不在云端构建。

本地双端口示例，从仓库根目录运行：

```sh
python3 -m http.server 8191 --bind 127.0.0.1 --directory services/server-forum/web/game-sandbox
VITE_GAME_SANDBOX_URL=http://127.0.0.1:8191/ VITE_GAME_RESOURCE_ORIGIN=https://chat.xiangban.xyz FORUM_API=https://chat.xiangban.xyz FORUM_WEB_PORT=5199 npm --prefix services/server-forum/web run dev -- --host 127.0.0.1
```

访问 `http://127.0.0.1:5199/packs/55`，该游戏来自真实社区。第二条命令中的开发专用资源源须与 API 返回的文件 URL 的 origin 一致；Vite 仅把该源 `/files/` 请求代理到 `FORUM_API`。生产不执行此重写。

论坛重新检查公开状态、审核等级及固定版本，匿名下载并校验长度、SHA-256、ZIP 和文本清单。宿主只通过精确来源、source 和 nonce 接收已验证文本；游戏在不授予同源权限的嵌套 iframe 内运行，CSP 禁止联网、表单、嵌套框架等。仅支持内联或根目录相对路径的普通 script 和 stylesheet；不支持模块脚本、联网、游戏存档、二进制及系统能力。浏览器必须支持 `DecompressionStream('deflate-raw')`。浏览器对 `navigate-to` CSP 支持不一致，不能将它作为阻断 iframe 自身导航的唯一防线；部署前需验证目标浏览器、部署头和隔离策略。不要在此站点设置任何敏感 cookie 或服务端能力。
