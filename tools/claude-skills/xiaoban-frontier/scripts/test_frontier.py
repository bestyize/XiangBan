import base64
import json
import os
import subprocess
import sys
import tempfile
import threading
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

SCRIPT = Path(__file__).with_name('frontier.py')

class Handler(BaseHTTPRequestHandler):
    requests = []
    def log_message(self, *args): pass
    def respond(self):
        data = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        self.requests.append((self.command, self.path, data))
        if self.path.endswith('/chat/completions'):
            result = {'choices': [{'message': {'content': json.dumps({'title':'标题','summary':'摘要','body':'正文'})}}]}
        elif self.path.endswith('/images/generations'):
            result = {'data': [{'b64_json': base64.b64encode(b'\x89PNG\r\n\x1a\nexample').decode()}]}
        elif self.path.endswith('/assets'):
            result = {'path': '/api/discover/frontier-assets/00000000-0000-4000-8000-000000000001'}
        else:
            result = {'revision': 12}
        body = json.dumps(result).encode()
        self.send_response(200); self.send_header('Content-Length', str(len(body))); self.end_headers(); self.wfile.write(body)
    do_POST = respond
    do_PUT = respond

class WorkflowTest(unittest.TestCase):
    def test_generate_and_submit_requires_cover(self):
        Handler.requests = []
        server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True); thread.start()
        try:
            with tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp); source = root / 'source.json'; output = root / 'result'
                source.write_text(json.dumps({'key':'example','title':'标题','source':'来源','url':'https://example.com/a','evidence':'已核实事实','image_prompt':'概念插画'}))
                env = {**os.environ, 'XIAOBAN_AI_URL': f'http://127.0.0.1:{server.server_port}', 'FORUM_URL': f'http://127.0.0.1:{server.server_port}', 'XIAOBAN_AI_TOKEN':'fake', 'FORUM_DISCOVER_TOKEN':'fake'}
                def run(*args): return subprocess.run([sys.executable, str(SCRIPT), *map(str,args)], env=env, capture_output=True, text=True)
                self.assertEqual(run('generate', source, output).returncode, 0)
                article = output / 'article.json'
                self.assertTrue((output / 'image-1.png').is_file())
                self.assertEqual(run('submit', article).returncode, 0)
                self.assertEqual(Handler.requests[-1][1], '/api/discover-editor/drafts/frontier/example')
                self.assertEqual(Handler.requests[-1][2]['expected_revision'], 0)
        finally:
            server.shutdown(); server.server_close(); thread.join()

if __name__ == '__main__': unittest.main()
