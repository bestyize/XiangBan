#!/usr/bin/env python3
"""把 build_release.py 生成的协议 2 包，原地替换线上同一作品的版本 1（不新增版本）。

只生成 SQL 和核对校验值，不连接任何服务器：
  python3 replace_version_sql.py --register /tmp/pack-out/register.json --zips /tmp/v2-stage-b --out /tmp/v2-stage-b/update.sql

作品编号写死在 PACK_IDS 里，只对应这两个案例；换 sha256、size、清单和文件键，保留审核状态、精选和署名。
"""
import argparse
import hashlib
import json
import sys
import zipfile
from pathlib import Path

PACK_IDS = {"ink-ferry": 56, "tea-sort": 57}


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--register", required=True, type=Path)
    parser.add_argument("--zips", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    statements = ["begin;"]
    for entry in json.loads(args.register.read_text(encoding="utf-8")):
        body = entry["body"]
        manifest = body["manifest"]
        slug = manifest["id"]
        if slug not in PACK_IDS:
            sys.exit(f"{slug} 不在允许替换的作品里")
        path = args.zips / f"{slug}-{manifest['version']}.zip"
        raw = path.read_bytes()
        if hashlib.sha256(raw).hexdigest() != body["sha256"] or len(raw) != body["size"]:
            sys.exit(f"{path.name} 与 register.json 的校验值不一致")
        if json.loads(zipfile.ZipFile(path).read("manifest.json")) != manifest:
            sys.exit(f"{path.name} 里的清单与 register.json 不一致")
        if manifest.get("runtimeProtocol") != 2 or manifest.get("schemaVersion") != 3:
            sys.exit(f"{slug} 不是协议 2 清单")
        text = json.dumps(manifest, ensure_ascii=False)
        if "$$" in text:
            sys.exit("清单里含有 $$，无法安全内联")
        statements.append(
            f"update pack_versions set label='{manifest['version']}', sha256='{body['sha256']}', size={body['size']}, "
            f"manifest=$${text}$$::jsonb, file_key='{body['file_key']}', runtime_protocol=2 "
            f"where pack_id={PACK_IDS[slug]} and version=1 and runtime_protocol=1;"
        )
    statements.append(
        "select pack_id, version, label, runtime_protocol, size, left(sha256,16) as sha, file_key "
        "from pack_versions where pack_id in (56,57) order by pack_id;"
    )
    statements.append("commit;")
    args.out.write_text("\n".join(statements) + "\n", encoding="utf-8")
    print(f"已核对两个包的校验值，写入 {len(statements)} 行 SQL 到 {args.out}")


if __name__ == "__main__":
    main()
