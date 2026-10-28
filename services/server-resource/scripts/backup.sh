#!/usr/bin/env bash
# 资源机异地备份：把 packs/ 和 covers/ 追加式同步到备份机，并核对校验清单。
#
# 内容包和封面的文件键不会复用，所以备份只增不删：主机上误删文件不会连带删掉备份。
# 用法：
#   RESOURCE_DATA_DIR=/data/resource BACKUP_TARGET=backup@host:/backup/xiaoban-resource scripts/backup.sh
#   scripts/backup.sh verify    # 核对备份端的文件与清单一致（恢复演练前先跑这个）
set -euo pipefail

DATA_DIR="${RESOURCE_DATA_DIR:-services/server-resource/data}"
TARGET="${BACKUP_TARGET:?请设置 BACKUP_TARGET，例如 backup@host:/backup/xiaoban-resource 或本机目录}"
MANIFEST="SHA256SUMS"

# 目标在本机目录时直接用，在远端时走 ssh。
is_remote() { [[ "$TARGET" == *:* ]]; }
remote_run() {
  if is_remote; then ssh "${TARGET%%:*}" "$@"; else bash -c "$*"; fi
}
target_path() { if is_remote; then echo "${TARGET#*:}"; else echo "$TARGET"; fi; }

sums=""
cleanup() { [ -n "$sums" ] && rm -f "$sums"; return 0; }
trap cleanup EXIT

backup() {
  [ -d "$DATA_DIR" ] || { echo "数据目录不存在：$DATA_DIR" >&2; exit 1; }
  sums="$(mktemp)"
  # 先生成本机清单：之后在备份端按它核对，才能发现传输中的损坏。
  (cd "$DATA_DIR" && find packs covers -type f 2>/dev/null | LC_ALL=C sort | xargs -r sha256sum) > "$sums"
  remote_run "mkdir -p '$(target_path)'"
  # 不带 --delete：只增不删。
  rsync -a --partial --ignore-existing --relative "$DATA_DIR/./packs" "$DATA_DIR/./covers" "$TARGET/" 2>/dev/null \
    || rsync -a --partial --ignore-existing --relative "$DATA_DIR/./packs" "$TARGET/"
  # 备份端的清单只增不删：和旧清单按路径合并（新的覆盖旧的），主机上删掉的文件仍然在清单里被核对。
  local old merged
  old="$(mktemp)"; merged="$(mktemp)"
  rsync -a "$TARGET/$MANIFEST" "$old" 2>/dev/null || : > "$old"
  awk '{ path = substr($0, 67); lines[path] = $0 } END { for (p in lines) print lines[p] }' "$old" "$sums" | LC_ALL=C sort -k2 > "$merged"
  rsync -a "$merged" "$TARGET/$MANIFEST"
  rm -f "$old" "$merged"
  echo "已备份 $(wc -l < "$sums") 个文件到 $TARGET，清单共 $(remote_run "wc -l < '$(target_path)/$MANIFEST'") 项"
}

verify() {
  # 在备份端核对：清单里的每个文件都存在且哈希一致。
  remote_run "cd '$(target_path)' && sha256sum --quiet -c $MANIFEST" \
    && echo "备份核对通过" || { echo "备份核对失败：有文件缺失或损坏" >&2; exit 1; }
}

case "${1:-backup}" in
  backup) backup ;;
  verify) verify ;;
  *) echo "用法：$0 [backup|verify]" >&2; exit 2 ;;
esac
