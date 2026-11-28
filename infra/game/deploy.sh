#!/usr/bin/env bash
set -euo pipefail

# 小伴游戏厅（game.xiaban.xyz）本地构建 + 静态产物同步脚本
# 遵守云服务器部署规范：100% 本地测试与构建，严禁在远端服务器执行 npm build。
#
# 用法示例：
#   HK_HOST=ubuntu@<香港机IP> ./infra/game/deploy.sh
# 可选变量：
#   REMOTE_DIR=/var/www/game.xiaban.xyz
#   SSH_PORT=22
#   SSH_KEY=~/.ssh/id_xxx

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APP_DIR="${ROOT_DIR}/client/xiaoban-games-h5"
HK_HOST="${HK_HOST:-}"
REMOTE_DIR="${REMOTE_DIR:-/var/www/game.xiaban.xyz}"
SSH_PORT="${SSH_PORT:-22}"
SSH_KEY="${SSH_KEY:-}"

if [[ -z "${HK_HOST}" ]]; then
  echo "用法: HK_HOST=ubuntu@<香港机IP> [SSH_KEY=~/.ssh/id_xxx] $0" >&2
  exit 1
fi

SSH_OPTS=(-p "${SSH_PORT}" -o BatchMode=yes)
if [[ -n "${SSH_KEY}" ]]; then
  SSH_OPTS+=(-i "${SSH_KEY}" -o IdentitiesOnly=yes)
fi

echo "==> [1/3] 本地同步官方游戏包、运行单元测试与生产构建 (${APP_DIR})"
npm --prefix "${APP_DIR}" test
npm --prefix "${APP_DIR}" run build

echo "==> [2/3] 确保远端静态目录存在: ${HK_HOST}:${REMOTE_DIR}"
ssh "${SSH_OPTS[@]}" "${HK_HOST}" "sudo mkdir -p '${REMOTE_DIR}' && sudo chown -R \$(whoami):\$(whoami) '${REMOTE_DIR}'"

echo "==> [3/3] 同步本地 dist/ 产物至香港云服务器"
rsync -avz --delete -e "ssh ${SSH_OPTS[*]}" "${APP_DIR}/dist/" "${HK_HOST}:${REMOTE_DIR}/"

echo "✓ 部署完成！请访问 https://game.xiaban.xyz/ 验证"
