#!/usr/bin/env python3
"""Build deterministic, content-addressed official game archives locally."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import zipfile

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "client/xiaoban-desktop/public/game-packs"
MANIFEST = ROOT / "assets/game-packs/distribution.json"
ALLOWED = {".html", ".htm", ".js", ".mjs", ".css", ".json", ".svg", ".png", ".jpg", ".jpeg", ".webp", ".gif", ".mp3", ".ogg", ".wav", ".woff2", ".md", ".txt"}
SEGMENT = re.compile(r"^[A-Za-z0-9_@-][A-Za-z0-9._@-]*$")


def build(output: Path) -> dict:
    output.mkdir(parents=True, exist_ok=True)
    ids = json.loads((SOURCE / "catalog.json").read_text())["games"]
    if len(ids) != len(set(ids)) or set(ids) != {p.name for p in SOURCE.iterdir() if (p / "manifest.json").is_file()}:
        raise ValueError("catalog and game directories differ")
    games = []
    for game_id in ids:
        root = SOURCE / game_id
        manifest = json.loads((root / "manifest.json").read_text(encoding="utf-8-sig"))
        if manifest.get("id") != game_id or manifest.get("schemaVersion") != 3 or manifest.get("runtimeProtocol") != 2:
            raise ValueError(f"invalid manifest: {game_id}")
        files = []
        for path in sorted(root.rglob("*")):
            if path.is_symlink():
                raise ValueError(f"symbolic link in {game_id}: {path}")
            if not path.is_file() or ".test." in path.name:
                continue
            name = path.relative_to(root).as_posix()
            if len(name.split("/")) > 8 or not all(SEGMENT.fullmatch(part) for part in name.split("/")) or path.suffix.lower() not in ALLOWED:
                raise ValueError(f"unsupported package path: {game_id}/{name}")
            files.append((name, path))
        names = {name for name, _ in files}
        if len({name.lower() for name in names}) != len(names) or not {"manifest.json", manifest["entry"], manifest["icon"], manifest["cover"]} <= names:
            raise ValueError(f"missing or duplicate package files: {game_id}")
        source_hash = hashlib.sha256()
        for name, path in files:
            source_hash.update(name.encode("ascii") + b"\0" + hashlib.sha256(path.read_bytes()).digest())
        temporary = output / f"{game_id}.part"
        with zipfile.ZipFile(temporary, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=6) as archive:
            for name, path in files:
                info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
                info.create_system = 3
                info.external_attr = 0o100644 << 16
                info.compress_type = zipfile.ZIP_DEFLATED
                archive.writestr(info, path.read_bytes(), compresslevel=6)
        data = temporary.read_bytes()
        md5 = hashlib.md5(data).hexdigest()
        sha = hashlib.sha256(data).hexdigest()
        destination = output / f"{md5}.zip"
        temporary.replace(destination)
        games.append({"id": game_id, "kind": manifest["kind"], "url": f"https://res.xiangban.xyz/files/{md5}.zip", "sha256": sha, "bytes": len(data), "sourceSha256": source_hash.hexdigest()})
    return {"version": 1, "games": games}


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, default=MANIFEST)
    args = parser.parse_args()
    manifest = build(args.output)
    args.manifest.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    print(f"Built {len(manifest['games'])} games, {sum(g['bytes'] for g in manifest['games'])} compressed bytes")


if __name__ == "__main__":
    main()
