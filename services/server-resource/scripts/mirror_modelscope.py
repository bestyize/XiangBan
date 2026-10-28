#!/usr/bin/env python3
"""把官方大资源镜像到魔搭社区，并把镜像地址登记到论坛（按版本锁定）。

魔搭只做官方镜像：主地址在 200 Mbps 的资源机上，魔搭分担下载压力。客户端对每个来源都校验 sha256，
所以魔搭上的文件即使被改动也不会被安装；本脚本在登记前也会下载回来核对一次。

用法：
  MODELSCOPE_TOKEN=... FORUM_ADMIN_TOKEN=... python3 scripts/mirror_modelscope.py \\
      --forum https://chat.xiangban.xyz --repo xiaoban/community-packs --pack-id 12 [--version 3] [--dry-run]

依赖：`pip install modelscope`（提供 `modelscope upload` 命令）。
"""
import argparse
import hashlib
import json
import os
import subprocess
import sys
import tempfile
import urllib.request


def sha256_file(path: str) -> str:
    digest = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def repo_path(kind: str, slug: str, version: int) -> str:
    """魔搭仓库里的路径：按内容包与版本锁定，同一版本永远指向同一个文件。"""
    return f"{kind}/{slug}/v{version}.zip"


def mirror_url(repo: str, path: str, revision: str = "master") -> str:
    return f"https://modelscope.cn/models/{repo}/resolve/{revision}/{path}"


def fetch(url: str, dest: str, max_bytes: int) -> None:
    """下载到本地，超过声明大小就中止。"""
    with urllib.request.urlopen(url, timeout=120) as response, open(dest, "wb") as out:
        read = 0
        while chunk := response.read(1 << 20):
            read += len(chunk)
            if read > max_bytes:
                raise SystemExit("下载内容比记录的大，已中止")
            out.write(chunk)


def api(forum: str, method: str, path: str, token: str | None = None, body: dict | None = None):
    headers = {"content-type": "application/json"}
    if token:
        headers["authorization"] = f"Bearer {token}"
    data = json.dumps(body).encode() if body is not None else None
    request = urllib.request.Request(f"{forum.rstrip('/')}{path}", data=data, method=method, headers=headers)
    with urllib.request.urlopen(request, timeout=30) as response:
        return json.load(response)


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--forum", required=True)
    parser.add_argument("--repo", required=True, help="魔搭仓库，如 xiaoban/community-packs")
    parser.add_argument("--pack-id", type=int, required=True)
    parser.add_argument("--version", type=int)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    admin_token = os.environ.get("FORUM_ADMIN_TOKEN", "")
    if not args.dry_run and not (admin_token and os.environ.get("MODELSCOPE_TOKEN")):
        sys.exit("需要环境变量 FORUM_ADMIN_TOKEN 和 MODELSCOPE_TOKEN（或加 --dry-run）")

    query = f"?version={args.version}" if args.version else ""
    info = api(args.forum, "GET", f"/api/packs/{args.pack_id}/download{query}")
    detail = api(args.forum, "GET", f"/api/packs/{args.pack_id}")
    path = repo_path(info["kind"], detail["slug"], info["version"])
    url = mirror_url(args.repo, path)
    print(f"内容包 {detail['title']} v{info['label']}（{info['size']} 字节）→ {url}")

    with tempfile.TemporaryDirectory() as tmp:
        local = os.path.join(tmp, "pack.zip")
        fetch(info["url"], local, info["size"])
        if sha256_file(local) != info["sha256"]:
            sys.exit("从资源机下载的文件与记录的 sha256 不一致，已中止")
        if args.dry_run:
            print("--dry-run：校验通过，不上传")
            return
        subprocess.run(["modelscope", "upload", args.repo, local, path, "--token", os.environ["MODELSCOPE_TOKEN"]], check=True)
        # 上传后再从魔搭下载回来核对，确认镜像地址真的可用、内容一致。
        back = os.path.join(tmp, "back.zip")
        fetch(url, back, info["size"])
        if sha256_file(back) != info["sha256"]:
            sys.exit("魔搭上的文件与 sha256 不一致，未登记镜像")

    api(args.forum, "POST", f"/api/admin/packs/{args.pack_id}/mirrors", admin_token, {"version": info["version"], "name": "modelscope", "url": url})
    print("镜像已登记")


if __name__ == "__main__":
    main()
