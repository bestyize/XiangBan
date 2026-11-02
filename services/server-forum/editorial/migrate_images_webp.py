#!/usr/bin/env python3
"""Explicit operator migration of hosted static images; no model or publishing calls.

Run as root on the forum host: prepare, apply, then verify. Originals and the
per-image journal remain in the private migration directory for rollback.
"""
import argparse
import base64
import fcntl
import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import time
import urllib.error
import urllib.request
import uuid

from PIL import Image
import worker as w


def pg(query):
    result = subprocess.run(['docker', 'exec', '-i', 'wallpaper_postgres', 'psql', '-X',
        '-v', 'ON_ERROR_STOP=1', '-U', 'postgres', '-d', 'xiaban_forum', '-Atq'],
        input=query, text=True, capture_output=True)
    if result.returncode:
        # SQL may contain private image bytes; never print it or PostgreSQL's echo.
        raise RuntimeError('PostgreSQL migration command failed; transaction rolled back')
    return result.stdout.strip()


def rows(query):
    return json.loads(pg('SELECT coalesce(json_agg(r),\'[]\'::json) FROM (' + query + ') r;'))


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def inventory():
    files = rows("""SELECT i.id::text, i.mime, i.size,
        md5(row_to_json(i)::text) AS row_md5,
        (SELECT array_agg(d.external_id) FROM public.editor_draft_assets a
         JOIN public.editor_drafts d ON d.id=a.draft_id WHERE a.image_id=i.id) AS local_keys
        FROM public.images i WHERE NOT i.editor_pending AND i.mime IN ('image/png','image/jpeg') ORDER BY i.id""")
    assets = rows("""SELECT a.id::text,a.mime,octet_length(a.data) AS size,md5(a.data) AS data_md5,
        a.kind,a.article_key FROM public.frontier_assets a
        WHERE a.mime IN ('image/png','image/jpeg') AND EXISTS (
            SELECT 1 FROM public.discover_items i WHERE i.kind=a.kind AND i.key=a.article_key
            AND i.status='published' AND i.published_at<=now() AND position(a.id::text in i.data::text)>0)
        ORDER BY a.id""")
    return [dict(r, store='images') for r in files] + [dict(r, store='frontier_assets') for r in assets]


def encode(raw):
    if not 1 <= len(raw) <= 10 * 1024 * 1024:
        raise ValueError('Input image size exceeds migration limit')
    with Image.open(io.BytesIO(raw)) as image:
        if image.format not in ('PNG', 'JPEG') or getattr(image, 'is_animated', False):
            raise ValueError('Only static PNG/JPEG images are migrated')
        if not all(1 <= n <= 8192 for n in image.size) or image.width * image.height > 32_000_000:
            raise ValueError('Image dimensions exceed migration limit')
        image.verify()
    with Image.open(io.BytesIO(raw)) as image:
        image.load()
        mode = 'RGBA' if 'A' in image.getbands() or 'transparency' in image.info else 'RGB'
        output = io.BytesIO()
        image.convert(mode).save(output, format='WEBP', quality=82, method=4)
        encoded = output.getvalue()
        with Image.open(io.BytesIO(encoded)) as check:
            check.load()
            if check.format != 'WEBP' or check.size != image.size or len(encoded) > 5 * 1024 * 1024:
                raise ValueError('Converted image failed validation')
        return encoded


def named(folder, row, suffix):
    return folder / (row['store'] + '-' + str(uuid.UUID(row['id'])) + suffix)


def owned_write(path, raw, reference):
    w.atomic(path, raw)
    stat = reference.stat()
    os.chown(path, stat.st_uid, stat.st_gid)


def prepare(args):
    args.backup.mkdir(parents=True, exist_ok=True, mode=0o700)
    plan_file = args.backup / 'prepared.json'
    if plan_file.exists():
        raise RuntimeError('Prepared plan already exists; use a new backup directory')
    db = sqlite3.connect('file:' + str(args.root / 'editorial.sqlite3') + '?mode=ro', uri=True)
    published = {r[0] for r in db.execute("SELECT id FROM jobs WHERE status='published'")}
    db.close()
    plan = []
    for row in inventory():
        original = named(args.backup, row, '.original')
        output = named(args.backup, row, '.webp')
        if row['store'] == 'images':
            source = args.uploads / ('img-' + row['id'])
            raw = source.read_bytes()
            if len(raw) != row['size']:
                raise RuntimeError('Stored image size does not match its database row')
            stat = source.stat()
            row.update(uid=stat.st_uid, gid=stat.st_gid, mode=stat.st_mode & 0o777)
        else:
            raw = base64.b64decode(pg("SELECT replace(encode(data,'base64'),E'\\n','') FROM public.frontier_assets WHERE id='" + str(uuid.UUID(row['id'])) + "';"), validate=True)
            if hashlib.md5(raw).hexdigest() != row['data_md5']:
                raise RuntimeError('Article asset changed during preparation')
        encoded = encode(raw)
        w.atomic(original, raw)
        w.atomic(output, encoded)
        row.update(original_sha256=sha(raw), webp_sha256=sha(encoded), webp_size=len(encoded), local_covers=[])
        candidates = row.get('local_keys') or ([row['article_key']] if row['store'] == 'frontier_assets' else [])
        for key in candidates:
            if key not in published:
                continue
            cover = args.root / 'articles' / key / 'cover.png'
            if not cover.exists() or sha(cover.read_bytes()) != row['original_sha256']:
                raise RuntimeError('Published local cover differs from hosted asset; inspect before conversion')
            if (cover.parent / 'cover.webp').exists() or (cover.parent / 'cover-webp.json').exists():
                raise RuntimeError('Local cover already has a conversion; inspect before overwriting')
            row['local_covers'].append(key)
        plan.append(row)
        if len(plan) % 50 == 0:
            print(json.dumps({'prepared': len(plan)}), flush=True)
    result = {'prepared_at': w.stamp(), 'images': plan,
              'original_bytes': sum(r['size'] for r in plan), 'webp_bytes': sum(r['webp_size'] for r in plan)}
    w.atomic(plan_file, result)
    print(json.dumps({k: v for k, v in result.items() if k != 'images'}), flush=True)


def content_fingerprint():
    # All fields, including moderation and permissions, must remain identical.
    return {table: pg("SELECT md5(coalesce(string_agg(md5(row_to_json(r)::text),'' ORDER BY md5(row_to_json(r)::text)),'')) FROM public." + table + ' r;')
            for table in ('topics', 'posts', 'editor_drafts', 'discover_items', 'discover_drafts')}


def database_change(plan, backup, reverse=False):
    statements = ['BEGIN; DO $migration$ DECLARE changed integer; BEGIN']
    for row in plan:
        identifier = str(uuid.UUID(row['id']))
        if row['store'] == 'images':
            if reverse:
                sets = "mime='" + row['mime'] + "',size=" + str(row['size'])
                condition = "mime='image/webp' AND size=" + str(row['webp_size'])
            else:
                sets = "mime='image/webp',size=" + str(row['webp_size'])
                condition = "md5(row_to_json(i)::text)='" + row['row_md5'] + "'"
            statement = "UPDATE public.images i SET " + sets + " WHERE id='" + identifier + "' AND " + condition + ';'
        else:
            raw = named(backup, row, '.original' if reverse else '.webp').read_bytes()
            encoded = base64.b64encode(raw).decode()
            expected = hashlib.md5(named(backup, row, '.webp').read_bytes()).hexdigest() if reverse else row['data_md5']
            mime = row['mime'] if reverse else 'image/webp'
            statement = "UPDATE public.frontier_assets SET mime='" + mime + "',data=decode('" + encoded + "','base64') WHERE id='" + identifier + "' AND md5(data)='" + expected + "';"
        statements.append(statement + " GET DIAGNOSTICS changed = ROW_COUNT; IF changed<>1 THEN RAISE EXCEPTION 'asset changed concurrently'; END IF;")
    statements.append('END; $migration$; COMMIT;')
    pg('\n'.join(statements))


def replace_files(args, plan, reverse=False):
    for row in plan:
        if row['store'] != 'images':
            continue
        target = args.uploads / ('img-' + row['id'])
        temporary = target.with_name('.webp-migration-' + row['id'])
        shutil.copyfile(named(args.backup, row, '.original' if reverse else '.webp'), temporary)
        os.chmod(temporary, row['mode'])
        os.chown(temporary, row['uid'], row['gid'])
        os.replace(temporary, target)


def sync_local(args, plan, reverse=False):
    for row in plan:
        for key in row['local_covers']:
            folder = args.root / 'articles' / key
            if reverse:
                (folder / 'cover-webp.json').unlink(missing_ok=True)
                (folder / 'cover.webp').unlink(missing_ok=True)
                continue
            owned_write(folder / 'cover.webp', named(args.backup, row, '.webp').read_bytes(), folder / 'cover.png')
            record = {'converted_at': w.stamp(), 'original_sha256': row['original_sha256'],
                      'webp_sha256': row['webp_sha256'], 'asset_id': row['id'], 'store': row['store']}
            owned_write(folder / 'cover-webp.json', record, folder / 'cover.png')


def service(*args):
    subprocess.run(['systemctl', *args], check=True, capture_output=True)


def wait_for_forum():
    for _ in range(30):
        try:
            with urllib.request.urlopen('http://127.0.0.1:55010/api/categories', timeout=3) as response:
                if response.status == 200:
                    return
        except OSError:
            time.sleep(1)
    raise RuntimeError('Forum did not become ready')


def apply(args):
    if (args.backup / 'applied.json').exists():
        raise RuntimeError('Migration already applied; run verify instead')
    plan = json.loads((args.backup / 'prepared.json').read_text())['images']
    if not plan:
        raise RuntimeError('No prepared images')
    was_active = subprocess.run(['systemctl', 'is-active', '--quiet', 'xiaoban-editorial.timer']).returncode == 0
    service('stop', 'xiaoban-editorial.timer')
    try:
        with (args.root / 'worker.lock').open('r+') as lock:
            fcntl.flock(lock, fcntl.LOCK_EX)  # Let all existing paid requests finish.
            for row in plan:
                if sha(named(args.backup, row, '.original').read_bytes()) != row['original_sha256'] or sha(named(args.backup, row, '.webp').read_bytes()) != row['webp_sha256']:
                    raise RuntimeError('Prepared files changed')
                if row['store'] == 'images' and sha((args.uploads / ('img-' + row['id'])).read_bytes()) != row['original_sha256']:
                    raise RuntimeError('Live image changed since preparation')
            service('stop', 'server-forum')
            began = time.monotonic()
            committed = False
            try:
                before = content_fingerprint()
                replace_files(args, plan)
                database_change(plan, args.backup)
                committed = True
                sync_local(args, plan)
                if content_fingerprint() != before:
                    raise RuntimeError('Article state changed unexpectedly')
                w.atomic(args.backup / 'applied.json', {'applied_at': w.stamp(), 'count': len(plan), 'article_state_unchanged': True})
            except BaseException:
                replace_files(args, plan, reverse=True)
                if committed:
                    database_change(plan, args.backup, reverse=True)
                sync_local(args, plan, reverse=True)
                raise
            finally:
                service('start', 'server-forum')
            wait_for_forum()
            print(json.dumps({'applied': len(plan), 'switch_seconds': round(time.monotonic()-began, 2)}), flush=True)
    finally:
        if was_active:
            service('start', 'xiaoban-editorial.timer')


def verify(args):
    plan = json.loads((args.backup / 'prepared.json').read_text())['images']
    checked = 0
    for row in plan:
        prefix = '/api/images/' if row['store'] == 'images' else '/api/discover/frontier-assets/' if row['kind'] == 'frontier' else '/api/discover/article-assets/'
        url = 'http://127.0.0.1:55010' + prefix + row['id']
        with urllib.request.urlopen(url, timeout=30) as response:
            raw = response.read(5 * 1024 * 1024 + 1)
            if response.headers.get_content_type() != 'image/webp' or sha(raw) != row['webp_sha256'] or len(raw) != row['webp_size']:
                raise RuntimeError('HTTP image read-back did not match converted bytes')
        for key in row['local_covers']:
            if sha(w.cover_file(args.root / 'articles' / key).read_bytes()) != row['webp_sha256']:
                raise RuntimeError('Local acceptance cover differs from served image')
        checked += 1
        if checked % 100 == 0:
            print(json.dumps({'verified': checked}), flush=True)
        time.sleep(0.03)
    remaining = inventory()
    result = {'verified_at': w.stamp(), 'converted': checked, 'verified': checked,
              'original_bytes': sum(r['size'] for r in plan), 'webp_bytes': sum(r['webp_size'] for r in plan),
              'local_editorial_covers': sum(len(r['local_covers']) for r in plan),
              'remaining_static_png_jpeg': len(remaining), 'article_state_unchanged': True}
    w.atomic(args.backup / 'verified.json', result)
    print(json.dumps(result), flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=('prepare', 'apply', 'verify'))
    parser.add_argument('--root', type=Path, default=Path('/var/lib/xiaoban-editorial'))
    parser.add_argument('--uploads', type=Path, default=Path('/home/ubuntu/xiangban-forum/uploads'))
    parser.add_argument('--backup', type=Path, required=True)
    args = parser.parse_args()
    globals()[args.command](args)
