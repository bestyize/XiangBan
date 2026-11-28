#!/usr/bin/env bash
set -euo pipefail

# 小伴产品矩阵总站与双端独立下载页（xiaban.xyz）本地构建 + 资源机 WebP 同步 + 香港机静态部署脚本
# 遵守云服务器部署规范：100% 本地构建测试，严禁在远端服务器执行 npm build。
#
# 用法示例：
#   HK_HOST=ubuntu@43.155.99.113 RES_HOST=ubuntu@124.220.64.203 SSH_KEY=~/.ssh/id_aihehua ./infra/portal/deploy.sh

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APP_DIR="${ROOT_DIR}/client/xiaban-portal"
ASSETS_SRC="${ROOT_DIR}/services/server-xiaoban/docs/design-specs/prototypes/assets"
HK_HOST="${HK_HOST:-}"
RES_HOST="${RES_HOST:-}"
REMOTE_DIR="${REMOTE_DIR:-/var/www/xiaban.xyz}"
RES_DIR="${RES_DIR:-/var/www/res.xiangban.xyz/img/xiaban-portal}"
SSH_PORT="${SSH_PORT:-22}"
SSH_KEY="${SSH_KEY:-}"

PORTAL_IMAGES=(
  app-icon-new.webp
  assistant.webp
  desktop-apps.webp
  desktop-companion.webp
  desktop-config-agent.webp
  desktop-config-ai.webp
  desktop-dragon.webp
  desktop-home.webp
  desktop-plugins-v318.webp
  desktop-wallpapers.webp
  desktop-widgets.webp
  mobile-ai-chat.webp
  mobile-ai-home.webp
  mobile-chat-build.webp
  mobile-community-new.webp
  mobile-my-works.webp
  site-ai-subscriptions.webp
  site-community.webp
  site-game.webp
  site-people.webp
  site-wallpaper.webp
  work-dusk-glider.webp
  work-ink-blade.webp
)

PLAZA_IMAGES=(
  water-sort.webp
  block-puzzle.webp
  dusk-glider.webp
  ink-blade.webp
  pro-cube.webp
  painting-adventure.webp
  remote-control.webp
  quick-launch.webp
  live-wallpapers.webp
  mbti.webp
)

if [[ -z "${HK_HOST}" ]]; then
  echo "用法: HK_HOST=ubuntu@<香港机IP> [RES_HOST=ubuntu@<资源机IP>] [SSH_KEY=~/.ssh/id_aihehua] $0" >&2
  exit 1
fi

SSH_OPTS=(-F /dev/null -p "${SSH_PORT}" -o BatchMode=yes)
if [[ -n "${SSH_KEY}" ]]; then
  SSH_OPTS+=(-i "${SSH_KEY}" -o IdentitiesOnly=yes)
fi

echo "==> [1/3] 本地运行测试与生产构建 (${APP_DIR})"
npm --prefix "${APP_DIR}" test
npm --prefix "${APP_DIR}" run build

if [[ -n "${RES_HOST}" ]]; then
  echo "==> [2/3] 同步 ${#PORTAL_IMAGES[@]} 张主图与 ${#PLAZA_IMAGES[@]} 张游乐场图标至资源机 ${RES_HOST}:${RES_DIR}"
  ssh "${SSH_OPTS[@]}" "${RES_HOST}" "mkdir -p '${RES_DIR}/plaza'"
  IMG_PATHS=()
  for name in "${PORTAL_IMAGES[@]}"; do
    IMG_PATHS+=("${ASSETS_SRC}/${name}")
  done
  rsync -avz --no-perms --no-owner --no-group -e "ssh ${SSH_OPTS[*]}" "${IMG_PATHS[@]}" "${RES_HOST}:${RES_DIR}/"
  PLAZA_PATHS=()
  for name in "${PLAZA_IMAGES[@]}"; do
    PLAZA_PATHS+=("${ASSETS_SRC}/plaza/${name}")
  done
  rsync -avz --no-perms --no-owner --no-group -e "ssh ${SSH_OPTS[*]}" "${PLAZA_PATHS[@]}" "${RES_HOST}:${RES_DIR}/plaza/"
else
  echo "==> [2/3] 未指定 RES_HOST，跳过资源机图片同步"
fi

echo "==> [3/3] 同步本地 dist/ 产物至香港机 ${HK_HOST}:${REMOTE_DIR}"
ssh "${SSH_OPTS[@]}" "${HK_HOST}" "sudo mkdir -p '${REMOTE_DIR}' && sudo chown -R \$(whoami):\$(whoami) '${REMOTE_DIR}'"
rsync -avz --delete -e "ssh ${SSH_OPTS[*]}" "${APP_DIR}/dist/" "${HK_HOST}:${REMOTE_DIR}/"

echo "✓ 部署完成！请访问 https://xiaban.xyz/ 、/desktop 、/app 验证"
