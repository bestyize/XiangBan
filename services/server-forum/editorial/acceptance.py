#!/usr/bin/env python3
"""Read-only campaign acceptance: signed reviews, real public articles and covers.

Runs without model calls or publishing credentials. Detailed results stay in the
private state directory; only the aggregate report is suitable for the repository.
"""
import argparse
from collections import Counter, defaultdict
import hashlib
import json
from pathlib import Path
import re
import sqlite3
import time
import urllib.parse
import urllib.request

import worker as w


def load(path):
    return json.loads(path.read_text())


def check(root, config):
    db = sqlite3.connect('file:' + str(root / 'editorial.sqlite3') + '?mode=ro', uri=True)
    db.row_factory = sqlite3.Row
    jobs = [dict(x) for x in db.execute("SELECT * FROM jobs WHERE status='published' ORDER BY agent,id")]
    db.close()
    agents = {a['id']: a for a in config['agents'] if a.get('enabled')}
    counts = Counter()
    issues = []
    titles, covers = defaultdict(list), defaultdict(list)
    for index, job in enumerate(jobs, 1):
        # Cover HEAD requests also consume the public site's IP quota. Leave
        # capacity for real readers and the independently running publisher.
        time.sleep(0.7)
        key, agent = job['id'], agents[job['agent']]
        folder = root / 'articles' / key
        try:
            article, review, evidence = [load(folder / f) for f in ['article.json', 'review.json', 'evidence.json']]
            w.validate_article(article)
            digest = hashlib.sha256(json.dumps(article, sort_keys=True, ensure_ascii=False).encode()).hexdigest()
            assert review.get('approved') is True and review.get('issues') == [], 'review not approved'
            assert review.get('article_sha256') == digest, 'review version mismatch'
            assert evidence['url'] == job['url'] and len(evidence['text']) >= 500, 'missing source evidence'
            raw = (folder / 'cover.png').read_bytes()
            w.validate_image(raw)
            covers[hashlib.sha256(raw).hexdigest()].append(key)
            titles[article['title'].strip()].append(key)
            prose = re.sub(r'```[^\n]*\n.*?```', '', article['body'], flags=re.S)
            assert not re.search(r'^\s*[A-Za-z]\s*$', prose, re.M), 'isolated prose character; inspect manually'
            kind = agent['destination']
            if kind == 'topic':
                public = w.api(config['forum_url'], '/api/topics/' + str(job['topic_id']))
                imported = load(folder / 'import.json')
                assert public['title'] == article['title'], 'public title changed'
                assert public['category']['slug'] == w.canonical_category(agent['category']), 'wrong public category'
                assert public['min_read_level'] == 0 and public.get('status') == 'normal', 'not public'
                assert public['images'] and public['body_html'], 'missing public image/body'
                cover = public['images'][0]
                expected = imported['body'].replace('](images/cover.png)', '](<images/cover.png>)')
                expected = expected.replace('](<images/cover.png>)', '](<' + cover + '>)')
                assert public['body_md'].strip() == expected.strip(), 'published body differs from approved import'
                assert article['body'].strip() in expected, 'approved article differs from import'
                assert evidence['url'] in public['body_md'], 'missing public source'
                assert public.get('help') is None, 'guide created a real help request'
            else:
                assert kind in ('news', 'frontier'), 'unexpected campaign destination'
                pathkind = 'daily' if kind == 'news' else 'frontier'
                public = w.api(config['forum_url'], '/api/discover/' + pathkind + '/' + key)
                imported = load(folder / 'import-discover.json')
                assert public['title'] == article['title'] and public['summary'] == imported['summary'], 'public metadata changed'
                assert public['url'] == evidence['url'] and public['cover_url'] == imported['cover_url'], 'public source/cover changed'
                assert len(public['body_html']) >= 1200, 'missing public body'
                assert article['body'].strip() in imported['body'], 'approved article differs from import'
                if evidence.get('source_published_at'):
                    assert w.discover_matches({'source_published_at': public['source_published_at']}, {'source_published_at': evidence['source_published_at']}), 'source date changed'
                cover = public['cover_url']
            url = urllib.parse.urljoin(config['forum_url'], cover)
            assert urllib.parse.urlsplit(url).netloc == urllib.parse.urlsplit(config['forum_url']).netloc, 'off-site cover'
            with w.HTTP.open(urllib.request.Request(url, method='HEAD'), timeout=30) as response:
                assert response.headers.get_content_type().startswith('image/'), 'cover not an image'
                assert int(response.headers.get('Content-Length', 0)) == len(raw), 'public cover size mismatch'
            counts[agent['id']] += 1
        except (AssertionError, OSError, ValueError, KeyError, w.Stop) as exc:
            issues.append({'job_id': key, 'agent': agent['id'], 'public_url': job['public_url'], 'issue': str(exc)[:250]})
        if index % 50 == 0:
            print(json.dumps({'checked': index, 'total': len(jobs), 'issues': len(issues)}), flush=True)
    for label, mapping in [('identical_titles', titles), ('identical_covers', covers)]:
        for keys in mapping.values():
            if len(keys) > 1:
                issues.append({'issue': label, 'jobs': keys})
    rows = []
    for key, agent in agents.items():
        own = [j for j in jobs if j['agent'] == key]
        kind = agent['destination']
        path = '/daily' if kind == 'news' else '/frontier' if kind == 'frontier' else '/c/' + agent['category']
        rows.append({'id': key, 'name': agent['name'], 'accepted': counts[key], 'target': 20,
                     'url': config['forum_url'] + path, 'sample_urls': [j['public_url'] for j in own[:2]]})
    result = {'generated_at': w.stamp(), 'total_published': len(jobs), 'total_accepted': sum(counts.values()),
              'all_targets_accepted': len(agents) == 41 and all(counts[k] >= 20 for k in agents) and not issues,
              'targets': rows, 'issues': issues,
              'checks': ['version-bound independent review', 'source evidence', 'public title/category/body',
                         'public cover type and size', 'valid local PNG', 'unique titles and cover bytes',
                         'no synthetic help requests', 'source publication dates for news and frontier']}
    w.atomic(root / 'acceptance.json', result)
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path('/var/lib/xiaoban-editorial'))
    parser.add_argument('--config', type=Path, default=Path(__file__).with_name('agents.json'))
    args = parser.parse_args()
    result = check(args.root, load(args.config))
    print(json.dumps({k: v for k, v in result.items() if k not in ('targets', 'issues')}, ensure_ascii=False))
    print(json.dumps({'issues': result['issues'][:15]}, ensure_ascii=False))
    raise SystemExit(0 if result['all_targets_accepted'] else 1)
