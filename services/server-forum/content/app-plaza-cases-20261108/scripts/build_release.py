#!/usr/bin/env python3
"""把 packs/ 下的作品打成社区资源包 ZIP，并生成登记用的 register.json。只用标准库。

用法：python3 scripts/build_release.py --out /tmp/plaza-release
输出：
  <out>/packs/game/<slug>-<version>.zip     资源机上的文件键 packs/game/...
  <out>/covers/<slug>-<version>.png         资源机上的文件键 covers/...
  <out>/register.json                       交给 publish_remote.py 的登记数据

这两款离线作品使用协议 2，仍按小型文本包打包。清单和入口位于 ZIP 根目录。
"""
import argparse
import hashlib
import html
import json
import re
import shutil
import sys
import zipfile
from pathlib import Path
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parent.parent
WEB_BRIDGE = ROOT.parent.parent / "web/runtime-v2/bridge.js"
FIXED_TIME = (2026, 11, 8, 0, 0, 0)
TEXT_EXT = {".html", ".js", ".css", ".json", ".svg", ".md", ".txt"}
MAX_FILES, MAX_FILE, MAX_TOTAL, MAX_ZIP, MAX_COVER = 40, 1024 * 1024, 4 * 1024 * 1024, 5 * 1024 * 1024, 2 * 1024 * 1024
SLUG = re.compile(r"^[a-z0-9][a-z0-9-]{1,39}$")


def fail(message: str) -> None:
    print(f"错误：{message}", file=sys.stderr)
    sys.exit(1)


def collect(src: Path) -> list[Path]:
    files = [p for p in src.iterdir() if p.is_file()]
    if not (src / "manifest.json").is_file():
        fail(f"{src.name} 缺少 manifest.json")
    if not (src / "index.html").is_file():
        fail(f"{src.name} 缺少入口 index.html")
    if len(files) > MAX_FILES:
        fail(f"{src.name} 文件数 {len(files)} 超过 {MAX_FILES}")
    total = 0
    for p in files:
        if p.suffix.lower() not in TEXT_EXT:
            fail(f"{src.name}/{p.name} 不是允许的文本文件类型")
        size = p.stat().st_size
        if size > MAX_FILE:
            fail(f"{src.name}/{p.name} 超过 1 MB")
        total += size
    if total > MAX_TOTAL:
        fail(f"{src.name} 文本合计超过 4 MB")
    return sorted(files, key=lambda p: (p.name != "manifest.json", p.name))


def build_zip(files: list[Path], dest: Path) -> None:
    dest.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(dest, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for path in files:
            info = zipfile.ZipInfo(path.name, FIXED_TIME)
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o644 << 16
            archive.writestr(info, path.read_bytes())


def stage_browser(files: list[Path], target: Path, parent_origin: str) -> None:
    if target.exists():
        fail(f"{target} 已存在，请使用新的浏览器预览输出目录")
    target.mkdir(parents=True)
    injection = f'<script src="./_xb_runtime.js" data-parent-origin="{html.escape(parent_origin, quote=True)}"></script>'
    for path in files:
        dest = target / path.name
        if path.suffix == ".html":
            source = path.read_text(encoding="utf-8")
            head = re.search(r"<head\b[^>]*>", source, re.IGNORECASE)
            if not head:
                fail(f"{path.name} 缺少 head，无法在作品脚本前注入 SDK")
            dest.write_text(source[:head.end()] + injection + source[head.end():], encoding="utf-8")
        else:
            shutil.copyfile(path, dest)
    shutil.copyfile(WEB_BRIDGE, target / "_xb_runtime.js")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--browser-out", type=Path)
    parser.add_argument("--browser-version", type=int)
    parser.add_argument("--forum-origin", default="https://chat.xiangban.xyz")
    args = parser.parse_args()
    origin = urlsplit(args.forum_origin)
    if args.browser_out and (
        not args.browser_version or args.browser_version <= 0 or not origin.hostname or origin.username or origin.password
        or origin.path or origin.query or origin.fragment
        or not (origin.scheme == "https" or (origin.scheme == "http" and origin.hostname in ("127.0.0.1", "localhost")))
    ):
        fail("浏览器预览需要有效的版本编号和论坛源")
    release = json.loads((ROOT / "release.json").read_text(encoding="utf-8"))
    entries = []
    for item in release["packs"]:
        slug = item["slug"]
        src = ROOT / "packs" / slug
        manifest = json.loads((src / "manifest.json").read_text(encoding="utf-8"))
        if manifest.get("id") != slug or not SLUG.match(slug):
            fail(f"{slug} 的清单 id 与目录名不一致或格式不对")
        if manifest.get("kind") != "game" or manifest.get("schemaVersion") != 3 or manifest.get("runtimeProtocol") != 2:
            fail(f"{slug} 的清单必须是协议 2、schemaVersion 3、kind game")
        files = collect(src)
        names = {p.name for p in files}
        for field in ("entry", "icon", "cover"):
            if manifest.get(field) not in names:
                fail(f"{slug} 的清单字段 {field} 指向缺失文件")
        version = manifest["version"]
        zip_key, cover_key = f"packs/game/{slug}-{version}.zip", f"covers/{slug}-{version}.png"
        zip_path = args.out / zip_key
        build_zip(files, zip_path)
        raw = zip_path.read_bytes()
        if len(raw) > MAX_ZIP:
            fail(f"{slug} 的 ZIP {len(raw)} 字节超过本示例的 5 MB 打包上限")
        with zipfile.ZipFile(zip_path) as archive:
            if archive.testzip() is not None or "manifest.json" not in archive.namelist():
                fail(f"{slug} 的 ZIP 校验失败")
        if args.browser_out:
            stage_browser(files, args.browser_out / str(item["pack_id"]) / f"v{args.browser_version}", args.forum_origin)
        cover_src = ROOT / item["cover_file"]
        cover = cover_src.read_bytes()
        if cover[:8] != b"\x89PNG\r\n\x1a\n" or len(cover) > MAX_COVER:
            fail(f"{item['cover_file']} 不是 2 MB 以内的 PNG")
        cover_dest = args.out / cover_key
        cover_dest.parent.mkdir(parents=True, exist_ok=True)
        cover_dest.write_bytes(cover)
        entries.append({
            "featured": bool(item.get("featured")),
            "body": {
                "kind": "game",
                "title": item["title"],
                "summary": item["summary"],
                "tags": item["tags"],
                "manifest": manifest,
                "file_key": zip_key,
                "sha256": hashlib.sha256(raw).hexdigest(),
                "size": len(raw),
                "cover_key": cover_key,
                "trust_level": release["trust_level"],
                "author_uid": release["author_uid"],
            },
        })
        print(f"{slug}: {len(raw)} 字节 sha256={entries[-1]['body']['sha256'][:16]}… 封面 {len(cover)} 字节")
    (args.out / "register.json").write_text(json.dumps(entries, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"已写入 {args.out / 'register.json'}")


if __name__ == "__main__":
    main()
