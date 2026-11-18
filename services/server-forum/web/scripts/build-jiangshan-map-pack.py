#!/usr/bin/env python3
"""Build the redistributable Jiangshan Atlas adaptation for Xiaoban V2."""

import argparse
import json
from pathlib import Path
import re
import shutil
import subprocess
import zipfile


UPSTREAM = "6d9c6fdd84b41520d3421eecedf9be7b0b6e6310"
WORK_ID = "jiangshan-map"
LAST_YEAR = 1948
WEB = Path(__file__).resolve().parents[1]


def read_assignment(text, marker):
    offset = text.index(marker) + len(marker)
    value, length = json.JSONDecoder().raw_decode(text[offset:])
    return value, lambda new: (
        text[:offset] + json.dumps(new, ensure_ascii=False, separators=(",", ":")) + text[offset + length:]
    )


def public_snapshots(text):
    atlas, serialize = read_assignment(text, "const D = ")
    snapshots = atlas["d"]
    restricted = {
        index
        for snapshot in snapshots.values() if isinstance(snapshot, dict)
        for index in snapshot.get("prefectures", [])
    }
    if not restricted:
        raise ValueError("Unexpected atlas format: no restricted prefecture entries")
    snapshots.pop("s_prc", None)
    features, rings = [], []
    feature_indices, ring_indices = {}, {}

    def ring(index):
        if index not in ring_indices:
            ring_indices[index] = len(rings)
            rings.append(atlas["r"][index])
        return ring_indices[index]

    def feature(index):
        if index not in feature_indices:
            item = atlas["f"][index].copy()
            shape = item.get("shape")
            if shape:
                if "_p" in shape:
                    item["shape"] = {"_p": [ring(i) for i in shape["_p"]]}
                elif "_m" in shape:
                    item["shape"] = {"_m": [[ring(i) for i in p] for p in shape["_m"]]}
                else:
                    raise ValueError("Unknown geometry encoding")
            feature_indices[index] = len(features)
            features.append(item)
        return feature_indices[index]

    for key, snapshot in snapshots.items():
        if isinstance(snapshot, list):
            snapshots[key] = [feature(i) for i in snapshot if i not in restricted]
            continue
        snapshot.pop("prefectures", None)
        snapshot.pop("prefRef", None)
        for layer in ("polities", "regions"):
            if layer in snapshot:
                snapshot[layer] = [feature(i) for i in snapshot[layer] if i not in restricted]
    return serialize({"r": rings, "f": features, "d": snapshots})


def pack_entry(source):
    html = (source / "index.html").read_text()
    html = re.sub(r'(?m)^\s*<link\b[^\n]*(?:https://|manifest\.webmanifest)[^\n]*>\n', '\n', html)
    html = re.sub(r'(?m)^\s*<script\b[^\n]*src="(?:https://|js/atlas_track\.js)[^\n]*\n', '\n', html)
    html = re.sub(r'(?s)\s*<script type="application/ld\+json">.*?</script>', '', html)
    html = html.replace('width=device-width, initial-scale=1', 'width=device-width, initial-scale=1, viewport-fit=cover')
    html = html.replace('中国历史地图 · 战国 → 今', '中国历史地图 · 前403—1948')
    html = html.replace('</head>', '<link rel="stylesheet" href="css/xiaoban.css">\n</head>')
    return html.replace('</body>', '<script src="js/xiaoban.js"></script>\n</body>')


def build(source, out):
    revision = subprocess.check_output(['git', '-C', source, 'rev-parse', 'HEAD'], text=True).strip()
    if revision != UPSTREAM:
        raise ValueError(f"Expected Jiangshan Atlas revision {UPSTREAM}, found {revision}")
    subprocess.run(['git', '-C', source, 'diff', '--quiet', 'HEAD', '--', 'index.html', 'LICENSE', 'css', 'js', 'assets'], check=True)

    player = out / WORK_ID
    player.mkdir(parents=True)
    for folder in ('js', 'css', 'assets'):
        (player / folder).mkdir()
    files = subprocess.check_output(['git', '-C', source, 'ls-files', '-z', 'js', 'assets/terrain', 'assets/icons'], text=True)
    for relative in files.rstrip('\0').split('\0'):
        path = source / relative
        if not relative.startswith('js/'):
            target = player / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(path, target)
            continue
        if path.name == 'atlas_track.js':
            continue
        data = path.read_text()
        if path.name in ('atlas_boot.js', 'atlas_snapshots.js'):
            data = public_snapshots(data)
        elif path.name == 'china_geo_data.js':
            data, count = re.subn(r'(?s)  const P = .*?  window\.PROVINCE_OUTLINES = P;', '  window.PROVINCE_OUTLINES = [];', data)
            if count != 1:
                raise ValueError("Could not remove the separately licensed modern province data")
        elif path.name == 'map_data.js':
            dynasties, serialize = read_assignment(data, 'window.DYNASTY_DATA = ')
            dynasties = [item for item in dynasties if item['fromYear'] <= LAST_YEAR]
            recent = dynasties[-1]
            recent['toYear'] = LAST_YEAR
            recent['milestones'] = [item for item in recent['milestones'] if item['year'] <= LAST_YEAR]
            data = serialize(dynasties)
        data = data.replace('中国历史地图 · 战国 → 今', '中国历史地图 · 前403—1948')
        (player / 'js' / path.name).write_text(data)

    shutil.copyfile(source / 'css/style.css', player / 'css/style.css')
    for name in ('terrain_relief.webp', 'favicon.svg'):
        shutil.copyfile(source / 'assets' / name, player / 'assets' / name)
    shutil.copyfile(source / 'LICENSE', player / 'LICENSE.txt')
    (player / 'index.html').write_text(pack_entry(source))
    adaptation = WEB / 'runtime-v2/jiangshan-map'
    shutil.copyfile(adaptation / 'xiaoban.css', player / 'css/xiaoban.css')
    shutil.copyfile(adaptation / 'xiaoban.js', player / 'js/xiaoban.js')
    shutil.copyfile(adaptation / 'SOURCE.md', player / 'SOURCE.md')
    shutil.copyfile(adaptation / 'icon.webp', player / 'icon.webp')
    shutil.copyfile(adaptation / 'cover.webp', player / 'cover.webp')
    manifest = {
        'schemaVersion': 3, 'runtimeProtocol': 2, 'kind': 'app', 'id': WORK_ID,
        'name': '江山时序', 'description': '沿时间轴看历代疆域，查地名，追踪历史人物行迹。',
        'version': '1.0.0-xb1', 'category': 'tool', 'entry': 'index.html',
        'icon': 'icon.webp', 'cover': 'cover.webp', 'orientation': 'any',
        'input': ['touch', 'mouse', 'keyboard'],
    }
    (player / 'manifest.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
    archive_path = out / 'Jiangshan-Map-v2.zip'
    with zipfile.ZipFile(archive_path, 'x', compression=zipfile.ZIP_DEFLATED, compresslevel=6) as archive:
        for path in sorted(player.rglob('*')):
            if path.is_file():
                entry = zipfile.ZipInfo(path.relative_to(player).as_posix())
                archive.writestr(entry, path.read_bytes(), compress_type=zipfile.ZIP_DEFLATED, compresslevel=6)
    print(f'{archive_path} ({archive_path.stat().st_size:,} bytes)')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    build(args.source.resolve(), args.out.resolve())
