#!/usr/bin/env bash
# 备份脚本的本机测试：同步、核对、篡改后核对失败、主机删文件后备份仍在。
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/data/packs/effect" "$work/data/covers"
printf 'pack one' > "$work/data/packs/effect/a.zip"
printf 'cover' > "$work/data/covers/c1"
export RESOURCE_DATA_DIR="$work/data" BACKUP_TARGET="$work/backup"

"$here/backup.sh"
"$here/backup.sh" verify
[ "$(cat "$work/backup/packs/effect/a.zip")" = "pack one" ]

# 主机上删掉文件，再备份：备份里的副本还在（只增不删）。
rm "$work/data/packs/effect/a.zip"
"$here/backup.sh"
[ -f "$work/backup/packs/effect/a.zip" ] || { echo "备份不应随主机删除而丢失"; exit 1; }

# 清单没有丢掉主机上已删除的文件，备份端的这个文件被破坏时核对必须能发现。
grep -q "packs/effect/a.zip" "$work/backup/SHA256SUMS" || { echo "清单应保留已删除文件"; exit 1; }
"$here/backup.sh" verify

# 备份端文件被篡改：核对必须失败。
printf 'tampered' > "$work/backup/covers/c1"
if "$here/backup.sh" verify 2>/dev/null; then echo "篡改后核对应当失败"; exit 1; fi
echo "backup tests ok"
