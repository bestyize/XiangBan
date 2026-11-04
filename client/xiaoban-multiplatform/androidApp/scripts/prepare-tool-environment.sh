#!/usr/bin/env bash
set -euo pipefail

output=$(realpath -m "${1:?output directory required}")
repo=${2:?GitHub repository required}
tag=${3:?GitHub release tag required}
work=$(mktemp -d)
container=
image=
cleanup() {
  if [[ -n "$container" ]]; then docker rm -f "$container" >/dev/null || true; fi
  if [[ -n "$image" ]]; then docker image rm "$image" >/dev/null || true; fi
  rm -rf "$work"
}
trap cleanup EXIT

fetch() {
  local url=$1 path=$2 digest=$3
  curl --fail --location --silent --show-error --retry 2 --connect-timeout 15 --max-time 180 "$url" -o "$path"
  printf '%s  %s\n' "$digest" "$path" | sha256sum --check --status
}

mkdir -p "$output/assets" "$output/jni/arm64-v8a" "$output/downloads"
ubuntu=ubuntu-base-24.04.5-base-arm64.tar.gz
node=node-v24.22.1-linux-arm64.tar.xz
fetch "https://cdimage.ubuntu.com/ubuntu-base/releases/24.04/release/$ubuntu" "$work/$ubuntu" \
  a91d5a93010193712d346d761372b7c9db6dfcf093893161c64ca107f05914f2
fetch "https://nodejs.org/dist/v24.22.1/$node" "$work/$node" \
  dbabf7a1cbf947e6059ffad2c27b80041db97dea1e553b62afdee7f13fca336e

termux=https://packages.termux.dev/apt/termux-main
while read -r path hash; do
  fetch "$termux/$path" "$work/${path##*/}" "$hash"
  dpkg-deb -x "$work/${path##*/}" "$work/termux"
done <<'PACKAGES'
pool/main/p/proot/proot_5.1.107.102_aarch64.deb 10a394b585c2fc2b2c8502544f4e5f1be581dc8bfff79e8ba62f16207423e030
pool/main/libt/libtalloc/libtalloc_2.5.0_aarch64.deb 556591f43bb773ad8777e1a29522640866a55f95dab71914418b94a8c58ad5a7
pool/main/liba/libandroid-shmem/libandroid-shmem_0.7_aarch64.deb 0da3a24d558b93c92bcf8d611e0826a99ff96e396b148e6cdf33b47c47c57ff6
PACKAGES

termux_root="$work/termux/data/data/com.termux/files/usr"
native="$output/jni/arm64-v8a"
install -m 755 "$termux_root/bin/proot" "$native/libproot.so"
install -m 755 "$termux_root/libexec/proot/loader" "$native/libproot-loader.so"
install -m 755 "$termux_root/lib/libtalloc.so.2.5.0" "$native/libtalloc.so"
install -m 755 "$termux_root/lib/libandroid-shmem.so" "$native/libandroid-shmem.so"
# Keep ELF offsets and segment alignment intact. Android extracts only lib*.so from the APK.
python3 - "$native/libproot.so" "$native/libtalloc.so" <<'PY'
from pathlib import Path
import sys

old = b'libtalloc.so.2\0'
new = b'libtalloc.so\0\0\0'
assert len(old) == len(new)
for name in sys.argv[1:]:
    path = Path(name)
    data = path.read_bytes()
    if old not in data:
        raise SystemExit(f'libtalloc ELF name not found in {name}')
    path.write_bytes(data.replace(old, new))
PY
mkdir -p "$output/assets/tool-environment-licenses"
for name in proot libtalloc libandroid-shmem; do
  if [[ -d "$termux_root/share/doc/$name" ]]; then
    license_dir="$output/assets/tool-environment-licenses/$name"
    mkdir -p "$license_dir"
    find "$termux_root/share/doc/$name" -type f -exec cp -t "$license_dir" {} +
  fi
done
fetch "https://github.com/termux/proot/archive/v5.1.107.102.zip" "$work/proot-source.zip" \
  30d12a6e2cc4147123756ff58edea262d3083bbc238c12d0ab9b39e3d1c40852
install -m 644 "$work/proot-source.zip" "$output/downloads/xiaoban-proot-5.1.107.102-source.zip"
unzip -p "$work/proot-source.zip" '*/COPYING' > "$output/assets/tool-environment-licenses/proot/COPYING"

image=$(docker import --platform linux/arm64 "$work/$ubuntu")
container=$(docker create --platform linux/arm64 --env DEBIAN_FRONTEND=noninteractive \
  --mount "type=bind,source=$work/$node,target=/tmp/node.tar.xz,readonly" "$image" /bin/sh -ec '
    apt-get update
    apt-get install -y --no-install-recommends ca-certificates git python3 python3-pip python3-venv xz-utils
    mkdir -p /opt/node
    tar -xJf /tmp/node.tar.xz -C /opt/node --strip-components=1
    PATH="/opt/node/bin:$PATH"
    node --version
    npm --version
    npx --version
    python3 --version
    python3 -m pip --version
    git --version
    apt-get clean
    rm -rf /var/lib/apt/lists/* /var/log/apt/* /var/log/dpkg.log
  ')
docker start -a "$container"
[[ "$(docker inspect --format '{{.State.ExitCode}}' "$container")" == 0 ]]
docker export "$container" | xz -T2 -6 > "$work/rootfs.tar.xz"
digest=$(sha256sum "$work/rootfs.tar.xz" | cut -d' ' -f1)
filename="xiaoban-tools-arm64-${digest:0:16}.tar.xz"
install -m 644 "$work/rootfs.tar.xz" "$output/downloads/$filename"
python3 - "$output" "$repo" "$tag" "$filename" "$digest" <<'PY'
import json
from pathlib import Path
import sys
from urllib.parse import quote

output, repo, tag, filename, sha256 = sys.argv[1:]
path = Path(output)
manifest = {
    "url": f"https://github.com/{repo}/releases/download/{quote(tag, safe='')}/{filename}",
    "bytes": (path / "downloads" / filename).stat().st_size,
    "sha256": sha256,
}
(path / "assets" / "tool-environment.json").write_text(json.dumps(manifest) + "\n")
PY
