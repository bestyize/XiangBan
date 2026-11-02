#!/usr/bin/env python3
"""Read-only campaign acceptance: signed reviews, real public articles and covers.

Runs without model calls or publishing credentials. Detailed results stay in the
private state directory; only the aggregate report is suitable for the repository.
"""
import argparse
from collections import Counter, defaultdict
import hashlib
from html.parser import HTMLParser
import json
from pathlib import Path
import re
import sqlite3
import struct
import time
import urllib.parse
import urllib.request
import zlib

import worker as w


def load(path):
    return json.loads(path.read_text())


def visible_body_key(value, html=False):
    """Compare the entire visible text, allowing Markdown rendering punctuation."""
    if html:
        class Text(HTMLParser):
            def __init__(self):
                super().__init__()
                self.parts = []
            def handle_data(self, data):
                self.parts.append(data)
        parser = Text()
        parser.feed(value)
        value = ''.join(parser.parts)
    else:
        value = re.sub(r'^ {0,3}(?:`{3,}|~{3,})[^\n]*$', '', value, flags=re.M)
        value = re.sub(r'^ {0,3}\d+[.)]\s+', '', value, flags=re.M)
        value = re.sub(r'\[([^\]]*)\]\([^)]*\)', r'\1', value)
    return re.sub(r'[\W_]+', '', value)


def check_png_integrity(raw):
    w.validate_image(raw)
    offset, image_data = 8, False
    while offset < len(raw):
        assert offset + 12 <= len(raw), 'truncated PNG chunk'
        length = struct.unpack('>I', raw[offset:offset + 4])[0]
        end = offset + 8 + length
        assert end + 4 <= len(raw), 'truncated PNG payload'
        assert zlib.crc32(raw[offset + 4:end]) & 0xffffffff == struct.unpack('>I', raw[end:end + 4])[0], 'PNG checksum mismatch'
        image_data |= raw[offset + 4:offset + 8] == b'IDAT'
        offset = end + 4
    assert image_data, 'PNG has no image data'


def check_cover_integrity(raw):
    w.validate_image(raw)  # Full decode for WebP as well as historical PNGs.
    if raw.startswith(b'\x89PNG\r\n\x1a\n'):
        check_png_integrity(raw)


def published_jobs(root):
    db = sqlite3.connect('file:' + str(root / 'editorial.sqlite3') + '?mode=ro', uri=True)
    db.row_factory = sqlite3.Row
    jobs = [dict(x) for x in db.execute("SELECT * FROM jobs WHERE status='published' ORDER BY agent,id")]
    db.close()
    return jobs


def check(root, config, follow=False):
    jobs = published_jobs(root)
    agents = {a['id']: a for a in config['agents'] if a.get('enabled')}
    counts = Counter()
    issues = []
    titles, covers = defaultdict(list), defaultdict(list)
    index = 0
    deadline, last_wait = time.monotonic() + 1800, 0
    while True:
        if index >= len(jobs):
            known = {j['id'] for j in jobs}
            jobs.extend(j for j in published_jobs(root) if j['id'] not in known)
            if index >= len(jobs):
                published = Counter(j['agent'] for j in jobs)
                if not follow or all(published[k] >= 20 for k in agents) or time.monotonic() >= deadline:
                    break
                if time.monotonic() - last_wait >= 55:
                    print(json.dumps({'checked': index, 'waiting_for_remaining_publications': sum(max(0, 20-published[k]) for k in agents)}), flush=True)
                    last_wait = time.monotonic()
                time.sleep(5)
                continue
        job = jobs[index]
        index += 1
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
            raw = w.cover_file(folder).read_bytes()
            check_cover_integrity(raw)
            covers[hashlib.sha256(raw).hexdigest()].append(key)
            titles[article['title'].strip()].append(key)
            prose = re.sub(r'```[^\n]*\n.*?```', '', article['body'], flags=re.S)
            assert not re.search(r'^\s*[A-Za-z]\s*$', prose, re.M), 'isolated prose character; inspect manually'
            kind = agent['destination']
            if kind == 'topic':
                public = w.api(config['forum_url'], '/api/topics/' + str(job['topic_id']))
                imported = load(folder / 'import.json')
                assert public['title'] == article['title'], 'public title changed'
                assert public['author']['uid'] == imported['author_uid'], 'public author changed'
                assert public['category']['slug'] == w.canonical_category(agent['category']), 'wrong public category'
                assert public['min_read_level'] == 0 and public.get('status') == 'normal', 'not public'
                assert public['images'] and public['body_html'], 'missing public image/body'
                cover = public['images'][0]
                image_path = imported['images'][0]['path']
                expected = imported['body'].replace('](' + image_path + ')', '](<' + image_path + '>)')
                expected = expected.replace('](<' + image_path + '>)', '](<' + cover + '>)')
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
                assert visible_body_key(imported['body']) == visible_body_key(public['body_html'], html=True), 'public article text differs from approved import'
                assert article['body'].strip() in imported['body'], 'approved article differs from import'
                assert evidence.get('source_published_at'), 'missing source publication date'
                assert w.discover_matches({'source_published_at': public['source_published_at']}, {'source_published_at': evidence['source_published_at']}), 'source date changed'
                cover = public['cover_url']
            url = urllib.parse.urljoin(config['forum_url'], cover)
            assert urllib.parse.urlsplit(url).netloc == urllib.parse.urlsplit(config['forum_url']).netloc, 'off-site cover'
            for attempt in range(3):
                try:
                    with w.HTTP.open(urllib.request.Request(url, method='HEAD'), timeout=30) as response:
                        assert response.headers.get_content_type().startswith('image/'), 'cover not an image'
                        assert int(response.headers.get('Content-Length', 0)) == len(raw), 'public cover size mismatch'
                    break
                except w.urllib.error.HTTPError as exc:
                    if exc.code not in (429, 502, 503, 504) or attempt == 2:
                        raise
                    time.sleep((2, 5)[attempt])
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
                         'public cover type and size', 'complete local image decoding; legacy PNG chunks and checksums', 'unique titles and cover bytes',
                         'no synthetic help requests', 'source publication dates for news and frontier']}
    w.atomic(root / 'acceptance.json', result)
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path('/var/lib/xiaoban-editorial'))
    parser.add_argument('--config', type=Path, default=Path(__file__).with_name('agents.json'))
    parser.add_argument('--follow', action='store_true', help='Include newly published articles and wait up to 30 minutes for the campaign')
    args = parser.parse_args()
    result = check(args.root, load(args.config), follow=args.follow)
    print(json.dumps({k: v for k, v in result.items() if k not in ('targets', 'issues')}, ensure_ascii=False))
    print(json.dumps({'issues': result['issues'][:15]}, ensure_ascii=False))
    raise SystemExit(0 if result['all_targets_accepted'] else 1)
