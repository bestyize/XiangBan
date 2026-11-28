#!/usr/bin/env bash
set -euo pipefail

# AI额度监控（aireset.xiaban.xyz）本地构建 + 静态产物同步脚本
# 遵守云服务器部署规范：100% 本地构建与自检，严禁在远端服务器执行 npm / 浏览器预渲染。
#
# 用法示例：
#   HK_HOST=ubuntu@43.155.99.113 SSH_KEY=~/.ssh/id_aihehua ./infra/aireset/deploy.sh
# 可选变量：
#   REMOTE_DIR=/var/www/aireset.xiaban.xyz
#   SSH_PORT=22

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APP_DIR="${ROOT_DIR}/client/aireset-web"
HK_HOST="${HK_HOST:-}"
REMOTE_DIR="${REMOTE_DIR:-/var/www/aireset.xiaban.xyz}"
SSH_PORT="${SSH_PORT:-22}"
SSH_KEY="${SSH_KEY:-}"

if [[ -z "${HK_HOST}" ]]; then
  echo "用法: HK_HOST=ubuntu@<香港机IP> [SSH_KEY=~/.ssh/id_xxx] $0" >&2
  exit 1
fi

# 必须显式写用户名并锁定密钥，避免 fail2ban 误封（见 ssh-server skill）。
if [[ "${HK_HOST}" != *@* ]]; then
  echo "HK_HOST 必须包含用户名，例如 ubuntu@43.155.99.113" >&2
  exit 1
fi

SSH_OPTS=(-p "${SSH_PORT}" -o BatchMode=yes)
if [[ -n "${SSH_KEY}" ]]; then
  SSH_OPTS+=(-i "${SSH_KEY}" -o IdentitiesOnly=yes)
fi

echo "==> [1/3] 本地构建、预渲染并自检 (${APP_DIR})"
(cd "${APP_DIR}" && npm test)

echo "==> [2/3] 确保远端静态目录存在: ${HK_HOST}:${REMOTE_DIR}"
ssh "${SSH_OPTS[@]}" "${HK_HOST}" "sudo mkdir -p '${REMOTE_DIR}' && sudo chown -R \$(whoami):\$(whoami) '${REMOTE_DIR}'"

echo "==> [3/3] 同步本地 dist/ 产物至香港云服务器"
rsync -avz --delete -e "ssh ${SSH_OPTS[*]}" "${APP_DIR}/dist/" "${HK_HOST}:${REMOTE_DIR}/"

echo "✓ 同步完成。请访问 https://aireset.xiaban.xyz/ 验证（首次上线需先完成 DNS 与 HTTPS 证书，见 infra/aireset/README.md）"
