#!/usr/bin/env python3
"""Freeze hash-verified ready-for-review cards; consumers must recheck hashes."""
import datetime
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent.parent
cards = json.loads((root / 'card-manifest.json').read_text(encoding='utf-8'))
ready = []
excluded = []
for card in cards['cards']:
    if card['status'] != 'ready_for_review':
        continue
    folder = root / card['original_key'].replace('/', '--')
    review_path = folder / 'review.json'
    try:
        review_bytes = review_path.read_bytes()
        if json.loads(review_bytes)['status'] != 'ready_for_review':
            raise ValueError('review state changed')
        required = [folder / name for name in ('README.source.md', 'README.zh-CN.md', 'README.en.md', 'metadata.json', 'review.json')]
        licenses = sorted(path for path in folder.glob('LICENSE*') if path.is_file())
        official_chinese_sources = [folder / name for name in ('README.official-zh.source.md', 'README.zh-CN.source.md') if (folder / name).is_file()]
        provenance_files = ([folder / name for name in ('description-index.json', 'translation-provenance.json')]
                            if card['original_key'] == 'punkpeye/awesome-mcp-servers' else [])
        if provenance_files:
            provenance_files += sorted((folder / 'assembly-inputs').glob('*.json'))
            if len(provenance_files) != 9:
                raise ValueError('awesome assembly inputs missing')
        if not licenses:
            raise ValueError('license file missing')
        files = {}
        for path in required + licenses + official_chinese_sources + provenance_files:
            content = path.read_bytes()
            if not content:
                raise ValueError(f'empty file: {path.name}')
            files[str(path.relative_to(root))] = hashlib.sha256(content).hexdigest()
        if review_path.read_bytes() != review_bytes:
            raise ValueError('review changed during snapshot')
        ready.append({'card': card, 'files_sha256': files})
    except (OSError, ValueError, KeyError, json.JSONDecodeError) as exc:
        excluded.append({'key': card['original_key'], 'reason': str(exc)})
result = {
    'inventory_revision': cards['revision'],
    'created_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'count': len(ready), 'excluded': excluded,
    'usage': '每次使用前重新校验每个文件 SHA-256；哈希不一致即舍弃该项目并重建快照。ready_for_review 仅表示待审。',
    'projects': ready,
}
tmp = root / '.ready-snapshot.json.tmp'
tmp.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
tmp.replace(root / 'ready-snapshot.json')
print({'ready_snapshot': len(ready), 'excluded': excluded})
