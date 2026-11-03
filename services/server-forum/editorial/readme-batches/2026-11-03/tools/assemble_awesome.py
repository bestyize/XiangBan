#!/usr/bin/env python3
"""Assemble reviewed translations only after every source description is present.

Writes draft README files but never changes review status; an editorial audit must
precede marking a project ready. IDs and source blobs are checked before output.
"""
from pathlib import Path
from collections import Counter
import hashlib
import json
import re

root = Path(__file__).resolve().parent.parent
p = root / 'punkpeye--awesome-mcp-servers'
raw = (p / 'README.source.md').read_bytes()
sha = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
index = json.loads((p / 'description-index.json').read_text())
assert sha == index['source_blob_sha'], 'source blob changed'
lines = raw.decode('utf-8').splitlines()
indexed = {x['id']: x for x in index['entries']}
assert len(indexed) == len(index['entries']) == 4632
translations = {}
provenance = {}
for filename in ['assembly-inputs/translation-fragments.json', 'assembly-inputs/awesome-part-self.json', 'assembly-inputs/awesome-part-a.json', 'assembly-inputs/awesome-part-b.json', 'assembly-inputs/awesome-part-tail.json', 'assembly-inputs/awesome-part-a-tail.json']:
    file = p / filename
    if not file.exists():
        print('missing fragment file:', filename)
        continue
    data = json.loads(file.read_text())
    if data.get('source_blob_sha'):
        assert data['source_blob_sha'] == sha, filename
    for row in data['entries']:
        text = row.get('zh') or row.get('chinese_description') or row.get('translated_description')
        if not text:
            continue
        key = row['id']
        if key in translations:
            raise ValueError(f'duplicate translation {key}: {filename}')
        ref = indexed[key]
        english = lines[ref['line'] - 1][ref['description_start']:]
        original = row.get('english') or row.get('english_description')
        if original != english:
            raise ValueError(f'source mismatch {key}: {filename}')
        for token in re.findall(r'`[^`]+`|\]\([^)]*\)', english):
            if token not in text:
                raise ValueError(f'missing literal {key}: {token}')
        translations[key] = text
        provenance[key] = {'id': key, 'source_line': ref['line'], 'batch': filename,
                           'translation_sha256': hashlib.sha256(text.encode('utf-8')).hexdigest()}
missing = set(indexed) - set(translations)
extra = set(translations) - set(indexed)
print(f'translated={len(translations)} missing={len(missing)} extra={len(extra)}')
if missing or extra:
    raise SystemExit('incomplete: ' + ', '.join(sorted(missing)[:10]))
nonitems = json.loads((p / 'assembly-inputs/awesome-nonitems.json').read_text())
assert nonitems['source_blob_sha'] == sha
nonitems = {int(k): v for k, v in nonitems['translated_lines'].items()}
assert not set(nonitems) & {v['line'] for v in indexed.values()}
byline = {v['line']: (k, v['description_start']) for k, v in indexed.items()}
commit = json.loads((p / 'metadata.json').read_text())['commit_sha']
base = f'https://github.com/punkpeye/awesome-mcp-servers/blob/{commit}/'
def pin_relative(text):
    def repl(m):
        link = m.group(2)
        if link.startswith(('#', 'https:', 'http:', 'mailto:', 'data:')):
            return m.group(0)
        return m.group(1) + base + link.removeprefix('./').lstrip('/') + m.group(3)
    return re.sub(r'(\]\()([^\s)]+)(\))', repl, text)
zh = []
en = []
for n, line in enumerate(lines, 1):
    en.append(pin_relative(line))
    if n in byline:
        key, start = byline[n]
        line = line[:start] + translations[key]
    else:
        line = nonitems.get(n, line)
    zh.append(pin_relative(line))
(p / 'README.zh-CN.md').write_text('\n'.join(zh) + '\n')
(p / 'README.en.md').write_text('\n'.join(en) + '\n')
(p / 'translation-provenance.json').write_text(json.dumps({
    'source_blob_sha': sha, 'count': len(provenance),
    'entries': [provenance[k] for k in sorted(provenance)],
}, ensure_ascii=False, indent=2) + '\n')
print('assembled; review status unchanged')
