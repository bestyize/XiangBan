#!/usr/bin/env python3
"""超级小伴论坛图文稿校验、打包、导入与发布。凭据只从环境变量读取。"""
import argparse
import base64
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import sys
from urllib.error import HTTPError
from urllib.parse import urlencode, urlparse
from urllib.request import Request, build_opener, HTTPRedirectHandler
import zipfile

try:
    import yaml
except ImportError:
    raise SystemExit('请先安装依赖：python -m pip install PyYAML>=6.0.2')

MAX_TOTAL = 80 * 1024 * 1024
MAX_FILE = 5 * 1024 * 1024

class StrictLoader(yaml.SafeLoader):
    def compose_node(self, parent, index):
        if self.check_event(yaml.AliasEvent):
            raise ValueError('元信息不支持 YAML 别名')
        return super().compose_node(parent, index)
    def construct_mapping(self, node, deep=False):
        result = {}
        for key, value in node.value:
            k = self.construct_object(key, deep=deep)
            if not isinstance(k, str) or k in result:
                raise ValueError('元信息字段必须唯一且为文字')
            result[k] = self.construct_object(value, deep=deep)
        return result
StrictLoader.add_constructor(yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG, StrictLoader.construct_mapping)

def safe_path(name):
    if name.startswith('./'):
        name = name[2:]
    if not name or len(name) > 300 or any(c in name for c in '\\\0:%?#') or name.startswith('/') or any(p in ('', '.', '..') for p in name.split('/')):
        raise ValueError(f'文件路径无效：{name}')
    return name

def allowed(name):
    return Path(name).suffix.lower() in ('.md', '.png', '.jpg', '.jpeg', '.gif', '.webp')

def read_files(path):
    path = Path(path)
    files = {}
    total = 0
    def add(name, content):
        nonlocal total
        name = safe_path(name)
        if name in files:
            raise ValueError(f'重复的文件路径：{name}')
        if not allowed(name):
            raise ValueError(f'不支持的文件：{name}')
        if len(content) > (120000 if name.lower().endswith('.md') else MAX_FILE):
            raise ValueError(f'文件过大：{name}')
        total += len(content)
        if total > MAX_TOTAL:
            raise ValueError('文件总大小不能超过 80 MB')
        files[name] = content
        if len(files) > 500:
            raise ValueError('每批最多 500 个文件')
    if path.is_symlink():
        raise ValueError('不支持符号链接')
    if path.is_dir():
        for f in sorted(path.rglob('*')):
            rel = f.relative_to(path).as_posix()
            if any(p.startswith('.') or p == '__MACOSX' for p in rel.split('/')):
                continue
            if f.is_symlink():
                raise ValueError(f'不支持符号链接：{rel}')
            if f.is_file():
                with f.open('rb') as stream:
                    add(rel, stream.read(MAX_FILE + 1))
    elif path.suffix.lower() == '.zip':
        if path.stat().st_size > 100 * 1024 * 1024:
            raise ValueError('ZIP 不能超过 100 MB')
        with zipfile.ZipFile(path) as z:
            if len(z.infolist()) > 1000:
                raise ValueError('ZIP 条目过多')
            for info in z.infolist():
                safe_path(info.filename.rstrip('/'))
                if info.is_dir() or any(p.startswith('.') or p == '__MACOSX' for p in info.filename.split('/')):
                    continue
                if stat.S_ISLNK(info.external_attr >> 16):
                    raise ValueError('ZIP 不能包含符号链接')
                with z.open(info) as stream:
                    add(info.filename, stream.read(MAX_FILE + 1))
    elif path.suffix.lower() == '.md':
        with path.open('rb') as stream:
            add(path.name, stream.read(120001))
    else:
        raise ValueError('请选择稿件目录、ZIP 或 Markdown 文件')
    return files

def parse_post(source):
    source = source.lstrip('\ufeff').replace('\r\n', '\n')
    match = re.fullmatch(r'---\n([\s\S]*?)\n---(?:\n|$)([\s\S]*)', source)
    if not match:
        raise ValueError('缺少 YAML 元信息')
    meta = yaml.load(match[1], Loader=StrictLoader)
    if not isinstance(meta, dict):
        raise ValueError('元信息需要是键值结构')
    extra = set(meta) - {'id', 'title', 'category', 'tags'}
    if extra:
        raise ValueError(f'不支持的字段：{sorted(extra)}')
    for key in ('id', 'title', 'category'):
        if not isinstance(meta.get(key), str) or not meta[key].strip():
            raise ValueError(f'缺少 {key}')
    if not re.fullmatch(r'[A-Za-z0-9_.-]{1,100}', meta['id']):
        raise ValueError('id 需要是 1–100 位字母、数字、点、下划线或连字符')
    tags = meta.get('tags', [])
    if not isinstance(tags, list) or any(not isinstance(t, str) for t in tags):
        raise ValueError('tags 需要是文字列表')
    title = ' '.join(meta['title'].split())
    if not 4 <= len(title) <= 80:
        raise ValueError('标题需为 4–80 字')
    if len(tags) > 5 or any(len(t.strip()) > 20 for t in tags):
        raise ValueError('最多 5 个标签，每个标签最多 20 字')
    body = match[2].strip()
    if len(body) > 20000:
        raise ValueError('正文不能超过 20000 字')
    return dict(external_id=meta['id'], title=title, category=meta['category'].strip(), tags=tags, body=body)

def build_posts(files):
    names = sorted(n for n in files if n.lower().endswith('.md'))
    if not 1 <= len(names) <= 50:
        raise ValueError('每批需要 1–50 篇 Markdown')
    result, seen = [], set()
    for name in names:
        try:
            post = parse_post(files[name].decode('utf-8-sig'))
            if post['external_id'] in seen:
                raise ValueError('同批稿件 id 重复')
            seen.add(post['external_id'])
            prefix = name.rsplit('/', 1)[0] + '/' if '/' in name else ''
            images = []
            for p, data in files.items():
                if p.startswith(prefix) and not p.lower().endswith('.md') and p[len(prefix):] in post['body']:
                    images.append(dict(path=p[len(prefix):], data_base64=base64.b64encode(data).decode()))
            if len(images) > 20 or sum(len(base64.b64decode(i['data_base64'])) for i in images) > 20 * 1024 * 1024:
                raise ValueError('每篇最多 20 张图片，合计不超过 20 MB')
            post['images'] = images
            result.append(dict(file=name, post=post))
        except (ValueError, yaml.YAMLError) as e:
            result.append(dict(file=name, error=str(e)))
    return result

class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        raise ValueError('接口发生重定向；请将 FORUM_URL 设置为最终 HTTPS 地址')

class Client:
    def __init__(self):
        self.base = os.environ.get('FORUM_URL', 'https://chat.xiangban.xyz').rstrip('/')
        u = urlparse(self.base)
        if (u.scheme != 'https' and not (u.scheme == 'http' and u.hostname in ('localhost', '127.0.0.1', '::1'))) or u.username or u.password or u.query or u.fragment or u.path not in ('', '/'):
            raise ValueError('FORUM_URL 必须是 HTTPS 站点根地址，本地开发可使用 localhost HTTP')
        self.token = os.environ.get('FORUM_EDITOR_TOKEN', '')
        if not self.token:
            raise ValueError('请先设置环境变量 FORUM_EDITOR_TOKEN')
        self.opener = build_opener(NoRedirect())
    def request(self, method, path, body=None, auth=True):
        req = Request(self.base + '/api' + path, method=method, data=None if body is None else json.dumps(body, ensure_ascii=False).encode(), headers={'Content-Type': 'application/json', **({'Authorization': 'Bearer ' + self.token} if auth else {})})
        try:
            with self.opener.open(req, timeout=45) as response:
                return json.load(response)
        except HTTPError as e:
            try:
                message = json.loads(e.read()).get('message', f'HTTP {e.code}')
            except (ValueError, AttributeError):
                message = f'HTTP {e.code}'
            raise ValueError(message) from None

def summarize(row):
    if 'error' in row:
        return dict(file=row['file'], ok=False, error=row['error'])
    p = row['post']
    return dict(file=row['file'], ok=True, external_id=p['external_id'], title=p['title'], category=p['category'], image_count=len(p['images']))

def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    commands.add_parser('context', help='读取实际板块、绑定作者与令牌权限')
    recent = commands.add_parser('recent', help='读取公开近期帖子，用于避免重复选题')
    recent.add_argument('--category', default='')
    recent.add_argument('--page', type=int, default=1)
    validate = commands.add_parser('validate', help='离线检查本地文件格式')
    validate.add_argument('path')
    pack = commands.add_parser('pack', help='生成供后台上传的 ZIP')
    pack.add_argument('path'); pack.add_argument('--output', required=True)
    imp = commands.add_parser('import', help='导入待发布稿，不自动发布')
    imp.add_argument('path'); imp.add_argument('--overwrite', action='store_true'); imp.add_argument('--report')
    publish = commands.add_parser('publish', help='发布选中的服务端稿件')
    publish.add_argument('--id', action='append', default=[]); publish.add_argument('--report')
    show = commands.add_parser('show', help='读取稿件并可导出预览 HTML')
    show.add_argument('id'); show.add_argument('--output')
    args = parser.parse_args(argv)
    if args.command in ('validate', 'pack', 'import'):
        files = read_files(args.path); rows = build_posts(files)
        if args.command == 'validate':
            out = [summarize(r) for r in rows]
        elif args.command == 'pack':
            if any('error' in r for r in rows):
                raise ValueError('存在格式错误，请先运行 validate')
            output = Path(args.output)
            if output.exists():
                raise ValueError('输出文件已存在，请使用新路径')
            output.parent.mkdir(parents=True, exist_ok=True)
            with zipfile.ZipFile(output, 'w', zipfile.ZIP_DEFLATED) as z:
                for name, data in files.items(): z.writestr(name, data)
            out = dict(output=str(output), posts=len(rows))
        else:
            client = Client(); context = client.request('GET', '/editor/context')
            author = context.get('author_uid')
            if not author: raise ValueError('请使用后台创建的专用创作令牌，不要使用管理员登录令牌')
            out = []
            for row in rows:
                result = summarize(row)
                if 'post' in row:
                    try:
                        draft = client.request('POST', '/editor/drafts', dict(**row['post'], author_uid=author, overwrite=args.overwrite))
                        result.update(id=draft['id'], status='imported')
                    except (ValueError, OSError) as e:
                        result.update(ok=False, error=str(e))
                out.append(result)
            if args.report:
                Path(args.report).write_text(json.dumps(out, ensure_ascii=False, indent=2), encoding='utf-8')
    elif args.command == 'context':
        out = Client().request('GET', '/editor/context')
    elif args.command == 'recent':
        out = Client().request('GET', '/topics?' + urlencode(dict(category=args.category, sort='new', page=max(1, args.page), page_size=30)), auth=False)
    elif args.command == 'show':
        from uuid import UUID
        identity = str(UUID(args.id))
        out = Client().request('GET', '/editor/drafts/' + identity)
        if args.output:
            Path(args.output).write_text('<!doctype html><meta charset="utf-8"><meta name="viewport" content="width=device-width"><style>body{max-width:800px;margin:40px auto;padding:20px;font:16px/1.8 system-ui}img{max-width:100%}pre{overflow:auto;background:#f5f5f5;padding:16px}</style>' + out['html'], encoding='utf-8')
            out = dict(id=identity, output=args.output)
    else:
        from uuid import UUID
        ids = list(args.id)
        if args.report:
            report = json.loads(Path(args.report).read_text(encoding='utf-8'))
            ids += [r['id'] for r in report if r.get('ok') and r.get('status') == 'imported']
        ids = list(dict.fromkeys(str(UUID(i)) for i in ids))
        if not ids: raise ValueError('请提供 --id，或包含已导入稿件的 --report')
        client = Client(); out = []
        for identity in ids:
            try: out.append(dict(draft_id=identity, ok=True, **client.request('POST', '/editor/drafts/' + identity + '/publish')))
            except (ValueError, OSError) as e: out.append(dict(draft_id=identity, ok=False, error=str(e)))
    print(json.dumps(out, ensure_ascii=False, indent=2))
    return int(isinstance(out, list) and any(not r.get('ok', True) for r in out))

if __name__ == '__main__':
    try: sys.exit(main())
    except (ValueError, OSError, yaml.YAMLError, zipfile.BadZipFile) as e:
        print(json.dumps(dict(ok=False, error=str(e)), ensure_ascii=False), file=sys.stderr)
        sys.exit(1)
