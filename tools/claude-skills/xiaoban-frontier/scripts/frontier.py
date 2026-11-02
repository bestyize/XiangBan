#!/usr/bin/env python3
"""Generate a reviewable frontier article with server-ai, or submit its JSON draft."""
import argparse
import base64
import json
import os
import re
import sys
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args, **kwargs):
        return None

opener = urllib.request.build_opener(NoRedirect())

def base_url(name, default):
    value = os.environ.get(name, default).rstrip('/')
    parsed = urllib.parse.urlsplit(value)
    if parsed.username or parsed.password or not parsed.hostname or parsed.query or parsed.fragment or parsed.path not in ('', '/'):
        raise ValueError(f'{name} 必须是无凭据的服务根地址')
    if parsed.scheme != 'https' and not (parsed.scheme == 'http' and parsed.hostname in ('localhost', '127.0.0.1', '::1')):
        raise ValueError(f'{name} 必须使用 HTTPS（本地测试除外）')
    return value

def call(base, path, token, data, method='POST'):
    req = urllib.request.Request(base + path, data=json.dumps(data, ensure_ascii=False).encode(), method=method,
        headers={'Authorization': 'Bearer ' + token, 'Content-Type': 'application/json'})
    try:
        with opener.open(req, timeout=180) as response:
            payload = response.read(50 * 1024 * 1024 + 1)
        if len(payload) > 50 * 1024 * 1024:
            raise ValueError('响应过大')
        return json.loads(payload)
    except urllib.error.HTTPError as exc:
        raise ValueError(f'服务请求失败：HTTP {exc.code}；请检查权限、额度和服务配置') from exc

def need(name):
    value = os.environ.get(name)
    if not value:
        raise ValueError(f'缺少环境变量 {name}')
    return value

def generate(args):
    source = json.loads(Path(args.source).read_text(encoding='utf-8'))
    for key in ('key', 'title', 'source', 'url', 'evidence', 'image_prompt'):
        if not isinstance(source.get(key), str) or not source[key].strip():
            raise ValueError(f'资料缺少 {key}')
    if not re.fullmatch(r'[a-z0-9_-]{1,60}', source['key']):
        raise ValueError('key 只能包含 1–60 位小写字母、数字、下划线和连字符')
    parsed = urllib.parse.urlsplit(source['url'])
    if parsed.scheme != 'https' or not parsed.hostname or parsed.username or parsed.password:
        raise ValueError('来源必须是无凭据的 HTTPS 地址')
    dest = Path(args.output)
    dest.mkdir(parents=True, exist_ok=True)
    if any((dest / name).exists() for name in ('article.json', 'image-1.png')):
        raise ValueError('输出文件已存在，请使用新目录，避免覆盖人工编辑')
    token = os.environ.get('XIAOBAN_AI_TOKEN')
    if not token:
        from auth_store import token as account_token
        token = account_token()
    base = base_url('XIAOBAN_AI_URL', 'https://ai.xiangban.xyz')
    prompts = [source['image_prompt']] + source.get('illustration_prompts', [])
    if not 1 <= len(prompts) <= 5 or not all(isinstance(x, str) and x.strip() for x in prompts):
        raise ValueError('配图提示词数量需为 1–5')
    markers = '、'.join(f'{{{{image:{i}}}}}' for i in range(1, len(prompts)))
    system = '你是小伴前沿编辑。仅依据给定资料写一篇 1800–2500 字的中文科技报道与深入解读；不得编造事实、体验、数字和引用，清楚标记观点及尚未确认的内容，文末附原始来源链接。返回 JSON 对象，字段 title、summary、body（Markdown）。'
    if markers: system += f'正文需在适当位置分别独占一行放置 {markers}，便于插入配图。'
    request = {'messages': [{'role': 'system', 'content': system},
                            {'role': 'user', 'content': json.dumps(source, ensure_ascii=False)}], 'stream': False, 'response_format': {'type': 'json_object'}}
    if os.environ.get('XIAOBAN_AI_TIER'):
        request['model'] = os.environ['XIAOBAN_AI_TIER']
    response = call(base, '/v1/chat/completions', token, request)
    content = response['choices'][0]['message']['content'].strip()
    content = re.sub(r'^```(?:json)?\s*|\s*```$', '', content, flags=re.I)
    try:
        article = json.loads(content)
    except json.JSONDecodeError:
        lines = content.splitlines()
        title = lines[0].lstrip('# ').strip() if lines else ''
        body = '\n'.join(lines[1:]).strip()
        summary = re.sub(r'[*#]', '', body.split('\n\n', 1)[0]).strip()[:300]
        article = {'title': title, 'summary': summary, 'body': body}
    if any(not isinstance(article.get(k), str) or not article[k].strip() for k in ('title', 'summary', 'body')):
        raise ValueError('模型返回的文章缺少必要字段')
    for index in range(1, len(prompts)):
        marker = f'{{{{image:{index}}}}}'
        if marker not in article['body']:
            sections = article['body'].split('\n\n')
            position = max(1, len(sections) * index // len(prompts))
            sections.insert(position, marker)
            article['body'] = '\n\n'.join(sections)
    image_files = []
    for index, prompt in enumerate(prompts):
        images = call(base, '/v1/images/generations', token, {'prompt': prompt, 'n': 1})
        image = images['data'][0].get('b64_json')
        if not image:
            raise ValueError('生图接口未返回 base64 图片')
        raw = base64.b64decode(image, validate=True)
        if not raw.startswith(b'\x89PNG\r\n\x1a\n') or len(raw) > 5 * 1024 * 1024:
            raise ValueError('图片须为不超过 5 MiB 的 PNG')
        filename = f'image-{index + 1}.png'
        (dest / filename).write_bytes(raw)
        image_files.append(filename)
    result = {'key': source['key'], 'title': article['title'], 'source': source['source'], 'url': source['url'],
              'summary': article['summary'], 'body': article['body'], 'cover_url': '', 'local_images': image_files}
    (dest / 'article.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(f'已生成待审核图文：{dest}。')

def submit(args):
    article = json.loads(Path(args.article).read_text(encoding='utf-8'))
    key = article.pop('key')
    images = article.pop('local_images', [])
    if not re.fullmatch(r'[a-z0-9_-]{1,60}', key):
        raise ValueError('无效 key')
    if len(images) > 5 or any(not re.fullmatch(r'image-[1-5]\.png', x) for x in images):
        raise ValueError('无效图片列表')
    for index in range(1, len(images)):
        if f'{{{{image:{index}}}}}' not in article['body']:
            raise ValueError(f'正文缺少插图位置 {index}')
    base = base_url('FORUM_URL', 'https://chat.xiangban.xyz')
    token = need('FORUM_DISCOVER_TOKEN')
    urls = []
    for image in images:
        raw = (Path(args.article).parent / image).read_bytes()
        result = call(base, f'/api/discover-editor/frontier/{key}/assets', token,
                      {'data_base64': base64.b64encode(raw).decode()})
        urls.append(base + result['path'])
    if urls:
        article['cover_url'] = urls[0]
        for index, url in enumerate(urls[1:], 1):
            marker = f'{{{{image:{index}}}}}'
            if marker not in article['body']:
                raise ValueError(f'正文缺少插图位置 {marker}')
            article['body'] = article['body'].replace(marker, f'![AI 生成的概念插图]({url})')
    data = {'expected_revision': args.revision, 'data': article, 'source_name': article['source'], 'source_url': article['url']}
    result = call(base_url('FORUM_URL', 'https://chat.xiangban.xyz'), '/api/discover-editor/drafts/frontier/' + key,
                  need('FORUM_DISCOVER_TOKEN'), data, method='PUT')
    print(f"待审稿已保存：{key}，revision={result['revision']}；尚未发布。")

def main():
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest='command', required=True)
    gen = sub.add_parser('generate'); gen.add_argument('source'); gen.add_argument('output'); gen.set_defaults(run=generate)
    send = sub.add_parser('submit'); send.add_argument('article'); send.add_argument('--revision', type=int, default=0); send.set_defaults(run=submit)
    args = parser.parse_args()
    try:
        args.run(args)
    except (ValueError, KeyError, IndexError, OSError, json.JSONDecodeError) as exc:
        print(f'失败：{exc}', file=sys.stderr)
        return 1
    return 0

if __name__ == '__main__':
    sys.exit(main())
