import contextlib
import importlib.util
import io
import json
import os
from pathlib import Path
import tempfile
import threading
import unittest
from unittest.mock import patch
from http.server import BaseHTTPRequestHandler, HTTPServer
import zipfile

spec = importlib.util.spec_from_file_location('forum_posts', Path(__file__).with_name('forum_posts.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
TEXT = '---\nid: demo-001\ntitle: 一篇可以发布的教程\ncategory: tech\ntags: [教程]\n---\n\n这里是正文，包含具体操作步骤和使用说明。\n'
PNG = b'\x89PNG\r\n\x1a\n' + b'example'

class PostsTest(unittest.TestCase):
    def test_zip_roundtrip_keeps_relative_images_and_reports_duplicate_ids(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); source = root / 'posts'; source.mkdir(); (source / 'images').mkdir()
            (source / 'one.md').write_text(TEXT + '\n![图](images/a.png)\n', encoding='utf-8')
            (source / 'images/a.png').write_bytes(PNG)
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(module.main(['pack', str(source), '--output', str(root / 'posts.zip')]), 0)
            rows = module.build_posts(module.read_files(root / 'posts.zip'))
            self.assertEqual(rows[0]['post']['images'][0]['path'], 'images/a.png')
            self.assertEqual(module.base64.b64decode(rows[0]['post']['images'][0]['data_base64']), PNG)
            files = module.read_files(source); files['two.md'] = files['one.md']
            self.assertIn('重复', module.build_posts(files)[1]['error'])
    def test_rejects_archive_escape_and_symlinks(self):
        with tempfile.TemporaryDirectory() as tmp:
            z = Path(tmp) / 'bad.zip'
            with zipfile.ZipFile(z, 'w') as f: f.writestr('../one.md', TEXT)
            with self.assertRaises(ValueError): module.read_files(z)
            with zipfile.ZipFile(z, 'w') as f:
                i = zipfile.ZipInfo('image.png'); i.create_system = 3; i.external_attr = 0o120777 << 16; f.writestr(i, '/etc/passwd')
            with self.assertRaises(ValueError): module.read_files(z)
    def test_yaml_cannot_silently_override_fields_or_use_aliases(self):
        for text in [TEXT.replace('id: demo-001', 'id: first\nid: second'), TEXT.replace('tags: [教程]', 'tags: &t [教程]\nother: *t')]:
            with self.assertRaises(ValueError): module.parse_post(text)
    def test_import_partial_failure_and_publish_report_do_not_duplicate_success(self):
        state = {'drafts': {}, 'topics': {}, 'post_calls': 0}
        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *args): pass
            def answer(self, code, body):
                raw = json.dumps(body).encode(); self.send_response(code); self.send_header('Content-Type', 'application/json'); self.end_headers(); self.wfile.write(raw)
            def do_GET(self): self.answer(200, {'author_uid': 'author', 'can_publish': True})
            def do_POST(self):
                body = json.loads(self.rfile.read(int(self.headers.get('Content-Length', '0'))) or b'{}')
                if self.path.endswith('/drafts'):
                    if body['external_id'] == 'bad': return self.answer(400, {'message': '板块不存在'})
                    identity = '00000000-0000-4000-8000-000000000001'; state['drafts'][body['external_id']] = identity
                    return self.answer(200, {'id': identity})
                identity = self.path.split('/')[-2]; state['topics'].setdefault(identity, 100); state['post_calls'] += 1
                self.answer(200, {'id': state['topics'][identity], 'url': 'https://example/t/100', 'already_published': state['post_calls'] > 1})
        server = HTTPServer(('127.0.0.1', 0), Handler); thread = threading.Thread(target=server.serve_forever, daemon=True); thread.start()
        try:
            with tempfile.TemporaryDirectory() as tmp, patch.dict(os.environ, {'FORUM_URL': f'http://127.0.0.1:{server.server_port}', 'FORUM_EDITOR_TOKEN': 'test-token'}):
                root = Path(tmp); posts = root / 'posts'; posts.mkdir(); (posts / 'one.md').write_text(TEXT, encoding='utf-8'); (posts / 'two.md').write_text(TEXT.replace('demo-001', 'bad'), encoding='utf-8')
                report = root / 'report.json'
                with contextlib.redirect_stdout(io.StringIO()):
                    self.assertEqual(module.main(['import', str(posts), '--report', str(report)]), 1)
                    self.assertFalse(state['topics'])
                    self.assertEqual(module.main(['publish', '--report', str(report)]), 0)
                    self.assertEqual(module.main(['publish', '--report', str(report)]), 0)
                self.assertEqual(len(state['topics']), 1)
                self.assertEqual(len(json.loads(report.read_text())), 2)
        finally: server.shutdown(); server.server_close(); thread.join()

if __name__ == '__main__': unittest.main()
