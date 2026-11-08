#!/usr/bin/env python3
"""在论坛所在机器上，用管理员账号登记作品并设置创作者精选。只用标准库，可重复运行。

管理员账号密码从论坛的 .env 里读取，不经过命令行，也不会被打印：
  python3 publish_remote.py --env /home/ubuntu/xiangban-forum/.env \
      --base http://127.0.0.1:55010 --register register.json

已存在的作品（同一个 id）不会重复登记，只会把精选状态调整到和 register.json 一致。
"""
import argparse
import json
import sys
import urllib.error
import urllib.request
from pathlib import Path


def read_env(path: Path) -> dict:
    values = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if line and not line.startswith("#") and "=" in line:
            key, _, value = line.partition("=")
            values[key.strip()] = value.strip().strip("'\"")
    return values


def call(base: str, method: str, path: str, token: str | None = None, body: dict | None = None):
    data = None if body is None else json.dumps(body).encode("utf-8")
    request = urllib.request.Request(base + path, data=data, method=method)
    request.add_header("Content-Type", "application/json")
    if token:
        request.add_header("Authorization", f"Bearer {token}")
    try:
        with urllib.request.urlopen(request, timeout=20) as response:
            return response.status, json.loads(response.read() or b"null")
    except urllib.error.HTTPError as error:
        try:
            return error.code, json.loads(error.read() or b"null")
        except ValueError:
            return error.code, None


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--env", required=True, type=Path)
    parser.add_argument("--base", required=True)
    parser.add_argument("--register", required=True, type=Path)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    entries = json.loads(args.register.read_text(encoding="utf-8"))
    if args.dry_run:
        for entry in entries:
            print("将登记", entry["body"]["manifest"]["id"], "精选" if entry["featured"] else "")
        return
    env = read_env(args.env)
    status, login = call(args.base, "POST", "/api/admin/login", body={
        "username": env.get("FORUM_ADMIN_USER", ""), "password": env.get("FORUM_ADMIN_PASSWORD", ""),
    })
    if status != 200 or not login or "token" not in login:
        sys.exit(f"管理员登录失败（HTTP {status}），已停止，没有登记任何作品")
    token = login["token"]
    status, listing = call(args.base, "GET", "/api/admin/packs?kind=game&page_size=100", token)
    existing = {item["slug"]: item for item in (listing or {}).get("items", [])} if status == 200 else {}
    for entry in entries:
        body, slug = entry["body"], entry["body"]["manifest"]["id"]
        if slug in existing:
            pack_id = existing[slug]["id"]
            print(f"{slug}: 已存在（id {pack_id}），跳过登记")
        else:
            status, result = call(args.base, "POST", "/api/admin/packs", token, body)
            if status != 200:
                sys.exit(f"{slug}: 登记失败（HTTP {status}）{result}")
            pack_id = result["id"]
            print(f"{slug}: 已登记，id {pack_id}")
        want = entry["featured"]
        if want or slug in existing:
            status, result = call(args.base, "POST", f"/api/admin/packs/{pack_id}/featured", token, {
                "featured": want, "note": "应用广场首发案例",
            })
            if status != 200:
                sys.exit(f"{slug}: 设置精选失败（HTTP {status}）{result}")
            print(f"{slug}: 精选={result['featured']}")


if __name__ == "__main__":
    main()
