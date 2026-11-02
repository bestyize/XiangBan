"""Persist and rotate the account session for unattended editorial work."""
import fcntl
import json
import os
import pathlib
import tempfile
import time
import urllib.request

ROOT = pathlib.Path.home() / '.config' / 'xiaoban-frontier'
SESSION = ROOT / 'session.json'

class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args, **kwargs):
        return None

def request(path, body):
    req = urllib.request.Request('https://account.xiangban.xyz' + path, data=json.dumps(body).encode(), headers={'Content-Type':'application/json'})
    with urllib.request.build_opener(NoRedirect()).open(req, timeout=30) as response:
        return json.load(response)

def save(response):
    ROOT.mkdir(parents=True, exist_ok=True, mode=0o700)
    os.chmod(ROOT, 0o700)
    data = {'access_token':response['access_token'], 'refresh_token':response['refresh_token'], 'expires_at':int(time.time()) + int(response['expires_in'])}
    fd, name = tempfile.mkstemp(prefix='.session-', dir=ROOT)
    try:
        os.fchmod(fd, 0o600)
        with os.fdopen(fd, 'w') as out:
            json.dump(data, out)
            out.flush()
            os.fsync(out.fileno())
        os.replace(name, SESSION)
    finally:
        if os.path.exists(name): os.unlink(name)

def token():
    ROOT.mkdir(parents=True, exist_ok=True, mode=0o700)
    lock = os.open(ROOT / 'session.lock', os.O_CREAT | os.O_RDWR, 0o600)
    with os.fdopen(lock, 'r+') as f:
        fcntl.flock(f, fcntl.LOCK_EX)
        if not SESSION.is_file():
            raise RuntimeError('缺少可刷新的运营会话，请先运行 login.py')
        session = json.loads(SESSION.read_text())
        if int(session['expires_at']) > time.time() + 60:
            return session['access_token']
        response = request('/api/auth/refresh', {'refresh_token':session['refresh_token'],'client_id':'xiaoban-frontier'})
        save(response)
        return response['access_token']
