#!/usr/bin/env bash
# 打包一个服务端镜像：按 infra/images/services.json 编译、装配构建上下文、构建并导出镜像。
# CI（BuildXiaoBanWallpaper 的“服务端镜像”任务）和本地用同一个脚本。
#
# 用法：infra/images/build-image.sh <服务编号> <输出目录> [镜像标签]
#   服务编号见 services.json（account、ai、forum、link、market、appstore、resource、wallpaper-collect、api、web）
#   镜像标签默认取当前提交的前 12 位
# 环境变量：
#   STAGE_ONLY=1       只编译并装配上下文，不调用 docker（本机没有 docker 时验证用）
#   NPM_CI_FLAGS       传给 npm ci 的额外参数（例如沙箱里用 --replace-registry-host=always --registry=https://registry.npmjs.org/）
# 输出（都在输出目录里）：
#   <镜像>-<标签>.tar.gz          docker save 的镜像包（docker load 即可）
#   <镜像>-<标签>-bundle.tar.gz   二进制 + 前端静态资源（不用 Docker 的主机，如 systemd 部署的论坛，直接替换文件用）；kind=docker 的服务没有
#   <编号>.meta.json              这个服务这次打包的信息，供汇总成 manifest.json
#   SHA256SUMS                    以上文件的校验和（追加写入）
set -euo pipefail

id="${1:?用法：build-image.sh <服务编号> <输出目录> [镜像标签]}"
out="${2:?缺少输出目录}"
repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo"
tag="${3:-$(git rev-parse --short=12 HEAD)}"
mkdir -p "$out"
out="$(cd "$out" && pwd)"

# 把清单里这个服务的字段读成 shell 变量。
eval "$(python3 - "$id" <<'PY'
import json, shlex, sys
services = json.load(open("infra/images/services.json", encoding="utf-8"))["services"]
s = next((x for x in services if x["id"] == sys.argv[1]), None)
if s is None:
    print("echo '没有这个服务：%s（可选：%s）' >&2; exit 2" % (sys.argv[1], "、".join(x["id"] for x in services)))
    sys.exit(0)
def put(k, v): print("%s=%s" % (k, shlex.quote("" if v is None else str(v))))
put("kind", s["kind"]); put("image", s["image"])
put("crate", s.get("crate")); put("bin", s.get("bin"))
put("web_dir", (s.get("web") or s.get("webEmbedded") or {}).get("dir")); put("web_embedded", "1" if "webEmbedded" in s else "")
put("dockerfile", s.get("dockerfile")); put("context", s.get("context"))
print("build_args=(%s)" % " ".join(shlex.quote("--build-arg=%s=%s" % kv) for kv in (s.get("buildArgs") or {}).items()))
PY
)"

docker_tags=(-t "$image:$tag" -t "$image:latest")
files=()

if [ "$kind" = "rust" ]; then
  if [ -n "$web_dir" ]; then
    # shellcheck disable=SC2086
    npm --prefix "$web_dir" ci --no-audit --no-fund ${NPM_CI_FLAGS:-}
    npm --prefix "$web_dir" run build
  fi
  cargo build --locked --release --manifest-path "$crate/Cargo.toml"
  ctx="$out/ctx-$id"
  rm -rf "$ctx" && mkdir -p "$ctx"
  cp "$crate/target/release/$bin" "$ctx/$bin"
  parts=("$bin")
  if [ -n "$web_dir" ] && [ -z "$web_embedded" ]; then
    mkdir "$ctx/web" && cp -r "$web_dir/dist/." "$ctx/web/"
    parts+=(web)
  fi
  cp "$crate/Dockerfile" "$ctx/Dockerfile"
  bundle="$image-$tag-bundle.tar.gz"
  tar -czf "$out/$bundle" -C "$ctx" "${parts[@]}"
  files+=("$bundle")
  if [ -z "${STAGE_ONLY:-}" ]; then
    docker build "${docker_tags[@]}" "$ctx"
  fi
elif [ "$kind" = "docker" ]; then
  if [ -z "${STAGE_ONLY:-}" ]; then
    docker build "${docker_tags[@]}" "${build_args[@]}" -f "$dockerfile" "$context"
  fi
else
  echo "未知的 kind：$kind" >&2; exit 2
fi

if [ -z "${STAGE_ONLY:-}" ]; then
  archive="$image-$tag.tar.gz"
  docker save "$image:$tag" "$image:latest" | gzip > "$out/$archive"
  files+=("$archive")
fi

# 校验和与这次打包的信息。
{ cd "$out" && for f in "${files[@]}"; do sha256sum "$f"; done; } >> "$out/SHA256SUMS"
python3 - "$id" "$image" "$tag" "$out" "${files[@]}" <<'PY'
import hashlib, json, pathlib, sys
sid, image, tag, out, *files = sys.argv[1:]
def sha(p): return hashlib.sha256(pathlib.Path(out, p).read_bytes()).hexdigest()
meta = {"id": sid, "image": image, "tag": tag, "files": [{"name": f, "sha256": sha(f)} for f in files]}
pathlib.Path(out, f"{sid}.meta.json").write_text(json.dumps(meta, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps(meta, ensure_ascii=False))
PY
