#!/usr/bin/env python3
"""Build an immutable, reviewable static release from a verified editorial snapshot.

Does not upload, publish the database catalog, or execute upstream content.
"""
import argparse
import re
from urllib.parse import urljoin
import datetime
import hashlib
import json
from pathlib import Path


def build(batch: Path, output: Path) -> dict:
    snapshot_bytes = (batch / 'ready-snapshot.json').read_bytes()
    snapshot = json.loads(snapshot_bytes)
    output.mkdir(parents=True, exist_ok=True)
    (output / 'projects').mkdir(exist_ok=True)
    index = {'schema': 1, 'released_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
             'inventory_revision': snapshot['inventory_revision'], 'projects': {}}
    skipped = []
    for item in snapshot['projects']:
        card = item['card']
        key = card['original_key']
        files = {}
        try:
            for rel, expected in item['files_sha256'].items():
                path = (batch / rel).resolve()
                if not path.is_relative_to(batch.resolve()):
                    raise ValueError('path outside batch')
                raw = path.read_bytes()
                if hashlib.sha256(raw).hexdigest() != expected:
                    raise ValueError('snapshot changed: ' + rel)
                files[rel] = raw
            folder = str(Path(card['readme_en_path']).parent)
            metadata = json.loads(files[folder + '/metadata.json'])
            review = json.loads(files[folder + '/review.json'])
            if review['status'] != 'ready_for_review':
                raise ValueError('review incomplete')
            source = files[folder + '/README.source.md']
            blob = metadata.get('readme_blob_sha') or metadata.get('readme', {}).get('blob_sha')
            actual_blob = hashlib.sha1(b'blob ' + str(len(source)).encode() + b'\0' + source).hexdigest()
            if not blob or blob != actual_blob:
                raise ValueError('upstream blob mismatch')
            zh = files[card['readme_zh_path']].decode('utf-8')
            en = files[card['readme_en_path']].decode('utf-8')
            translation = (metadata.get('translation_origin') or metadata.get('translation_source')
                           or metadata.get('translation_provenance') or metadata.get('translation_attribution')
                           or '非官方中文译文；依据项目官方 README 整理')
            if metadata.get('publish_official_zh'):
                official = metadata['official_zh_source']
                raw_zh = files[folder + '/README.official-zh.source.md']
                zh_blob = hashlib.sha1(b'blob ' + str(len(raw_zh)).encode() + b'\0' + raw_zh).hexdigest()
                if zh_blob != official['blob_sha']:
                    raise ValueError('official Chinese upstream blob mismatch')
                full_translation = zh.encode('utf-8')
                filename = hashlib.sha256(full_translation).hexdigest() + '.md'
                (output / 'projects' / filename).write_bytes(full_translation)
                translation = '中文页使用完整官方简体中文版，约收录 436 条；英文版收录 4632 条。'
                official_body = re.sub(
                    r'\]\((README[^ )]*\.md)\)',
                    lambda match: '](' + urljoin(official['source_url'], match[1]) + ')',
                    raw_zh.decode('utf-8'),
                )
                official_body = official_body.replace('](#什么是MCP？)', '](#什么是mcp)')
                zh = (
                    f'> {translation} [官方简体中文原文]({official["source_url"]}) · '
                    f'[查看社区全量中文译文](https://chat.xiangban.xyz/open-source-readmes/projects/{filename})\n\n'
                    + official_body
                )
            if not zh.strip() or not en.strip():
                raise ValueError('empty README')
            licenses = [{'name': Path(rel).name, 'text': raw.decode('utf-8')} for rel, raw in files.items()
                        if Path(rel).name.upper().startswith(('LICENSE', 'LICENCE', 'COPYING', 'NOTICE'))]
            if not licenses:
                raise ValueError('license missing')
            data = {'schema': 1, 'full_name': key, 'card': card,
                    'source_url': metadata.get('readme_source_url') or metadata.get('readme_url') or metadata.get('readme', {}).get('source_url'),
                    'commit': metadata.get('commit_sha'), 'fetched_at': metadata.get('fetched_at'),
                    'translation': translation,
                    'zh': zh, 'en': en, 'licenses': licenses}
            if not data['source_url'] or not data['commit']:
                raise ValueError('provenance missing')
            encoded = (json.dumps(data, ensure_ascii=False, separators=(',', ':')) + '\n').encode()
            digest = hashlib.sha256(encoded).hexdigest()
            target = 'projects/' + digest + '.json'
            (output / target).write_bytes(encoded)
            index['projects'][key.lower()] = target
        except (ValueError, KeyError, OSError) as error:
            skipped.append({'project': key, 'reason': str(error)})
    # Commit the index last: readers always see complete, immutable project payloads.
    temporary = output / '.index.json.tmp'
    temporary.write_text(json.dumps(index, ensure_ascii=False, indent=2) + '\n')
    temporary.replace(output / 'index.json')
    report = {'eligible': len(snapshot['projects']), 'released': len(index['projects']), 'skipped': skipped,
              'snapshot_created_at': snapshot['created_at'], 'snapshot_sha256': hashlib.sha256(snapshot_bytes).hexdigest()}
    (output / 'release-report.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('batch', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    print(json.dumps(build(args.batch, args.output), ensure_ascii=False, indent=2))
