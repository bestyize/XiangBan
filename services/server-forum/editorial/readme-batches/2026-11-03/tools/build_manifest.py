#!/usr/bin/env python3
"""Rebuild the batch status manifest and human-readable report from review files."""
import collections
import datetime
import json
from pathlib import Path

root = Path(__file__).resolve().parent.parent
inventory = json.loads((root / 'inventory.json').read_text(encoding='utf-8'))
projects = []
for index, item in enumerate(inventory['projects']):
    folder = root / item['full_name'].replace('/', '--')
    review_path = folder / 'review.json'
    review = json.loads(review_path.read_text(encoding='utf-8')) if review_path.exists() else {}
    projects.append({
        'index': index, 'key': item['full_name'], 'repository': item['full_name'],
        'directory': folder.name, 'status': review.get('status', 'pending'),
        'issues': review.get('issues', []),
    })
counts = collections.Counter(project['status'] for project in projects)
valid = {'ready_for_review', 'blocked', 'pending'}
if set(counts) - valid:
    raise ValueError(f'Unknown statuses: {set(counts) - valid}')
now = datetime.datetime.now(datetime.timezone.utc).isoformat()
manifest = {
    'source': 'inventory.json', 'revision': inventory['revision'], 'updated_at': now,
    'total': len(projects), 'ready_for_review': counts['ready_for_review'],
    'blocked': counts['blocked'], 'pending': counts['pending'],
    'collections': inventory['collections'], 'projects': projects,
}
manifest_tmp = root / '.manifest.json.tmp'
manifest_tmp.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
manifest_tmp.replace(root / 'manifest.json')
report = f'''# 开源精选 README 批次报告

数据来源：{inventory['source_url']}，目录 revision {inventory['revision']}；快照获取于 {inventory['fetched_at']}。报告生成于 {now}。

- 项目总数：{len(projects)}
- 可待审（ready_for_review）：{counts['ready_for_review']}
- 阻塞（blocked）：{counts['blocked']}
- 待处理（pending）：{counts['pending']}
- 专题：{len(inventory['collections'])}，保留原引用，详见 inventory.json。

逐项状态与问题见 manifest.json，项目卡片见 card-manifest.json。pending 包括尚未完成全文翻译、素材/许可核对或审校的项目，不能视为完成；ready_for_review 表示待审，不表示发布。现有目录与详情界面仍按旧导读字段工作，尚未提供经过约定和验证的中英文 README 全文字段及切换；批次文件未写入线上数据库或发布。

运行 `python3 tools/build_manifest.py` 和 `python3 tools/build_card_manifest.py` 可按现有 review.json 刷新快照；仅在采编工作停止写入时刷新，避免统计竞争。
'''
report_tmp = root / '.REPORT.md.tmp'
report_tmp.write_text(report, encoding='utf-8')
report_tmp.replace(root / 'REPORT.md')
print({key: manifest[key] for key in ('total', 'ready_for_review', 'blocked', 'pending')})
