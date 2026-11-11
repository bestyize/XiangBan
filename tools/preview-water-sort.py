"""Local viewport/lifecycle harness for the real, packaged Pour Joy game (no production SDK changes)."""
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlsplit, parse_qs

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / 'client/xiaoban-desktop/public/game-packs/water-sort'
ANDROID = ROOT / 'client/xiaoban-multiplatform/androidApp/src/main/builtin-runtime-v2'

PAGE = '''<!doctype html><meta charset="utf-8"><title>倒杯快乐 · 两端验收</title>
<style>body{margin:0;background:#e9dfd2;font:13px system-ui;color:#704c38}header{height:50px;display:flex;align-items:center;gap:15px;padding:0 20px}button,select{padding:8px;border:0;border-radius:8px;background:#fff8e9;color:#805637}main{display:flex;justify-content:center;padding:12px;width:max-content;min-width:100%;box-sizing:border-box}iframe{flex:none;border:1px solid #e2bd92;box-shadow:0 12px 30px #80532222;background:#fff2dc}#events{font-size:10px;max-width:250px}</style>
<header><b>倒杯快乐 · 实际游戏包</b><select id="size" aria-label="设备尺寸"><option value="390,844">手机 390 × 844</option><option value="360,740">小屏 360 × 740</option><option value="320,640">小屏 320 × 640</option><option value="844,390">横屏 844 × 390</option><option value="1200,800">桌面 1200 × 800</option></select><button id="hide">切后台</button><button id="show">回前台</button><span id="events"></span></header>
<main><iframe id="game" title="实际游戏"></iframe></main><script>
const frame=document.querySelector('iframe'),size=document.querySelector('#size');
function resize(){const [w,h]=size.value.split(',').map(Number);frame.style.width=w+'px';frame.style.height=h+'px';frame.src='/water-sort/index.html'+(w<1000?'?android=1':'');}
size.onchange=resize;resize();for(const name of ['hide','show'])document.getElementById(name).onclick=()=>frame.contentWindow.postMessage({qaEvent:name},location.origin);
addEventListener('message',e=>{if(e.source===frame.contentWindow&&e.data.qaLog)document.querySelector('#events').textContent=e.data.qaLog;});
</script>'''

SDK = '''<script>
(() => {const events=new Map();const landscape=innerWidth>innerHeight;const safe=landscape?{top:0,right:24,bottom:16,left:24}:{top:28,right:0,bottom:24,left:0};const capsule={top:safe.top+8,right:12,width:48,height:48};
for(const [k,v]of Object.entries(safe))document.documentElement.style.setProperty('--xb-safe-'+k,v+'px');for(const[k,v]of Object.entries(capsule))document.documentElement.style.setProperty('--xb-capsule-'+k,v+'px');
const log=s=>parent.postMessage({qaLog:s},location.origin);window.xb={info:{platform:'android',safeArea:safe,capsule,query:{},legacySave:null},ready:async()=>log('xb.ready'),on:(n,f)=>{if(!events.has(n))events.set(n,[]);events.get(n).push(f)},can:()=>false,storage:{get:async k=>JSON.parse(localStorage.getItem('qa-'+k)||'null'),set:async(k,v)=>localStorage.setItem('qa-'+k,JSON.stringify(v))},game:{start:async()=>log('game.start'),progress:async v=>log('progress '+v.score),result:async v=>log('result '+v.score)}};
addEventListener('message',e=>{if(e.source===parent&&e.origin===location.origin&&['hide','show'].includes(e.data?.qaEvent))for(const f of events.get(e.data.qaEvent)||[])f();});
addEventListener('DOMContentLoaded',()=>{const b=document.createElement('button');b.textContent='⋮';b.setAttribute('aria-label','模拟宿主菜单');b.id='qaCapsule';Object.assign(b.style,{position:'fixed',top:capsule.top+'px',right:capsule.right+'px',width:'48px',height:'48px',zIndex:1000,border:'0',background:'#fffa',borderRadius:'24px',fontSize:'25px'});b.onclick=()=>{for(const f of events.get('hide')||[])f();};document.body.append(b);});})();
</script><script src="/qa/builtin-runtime.js"></script><link rel="stylesheet" href="/qa/builtin-runtime.css">'''

class Handler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(ROOT / 'client/xiaoban-desktop/public/game-packs'), **kwargs)

    def send(self, data, mime):
        data = data.encode('utf-8') if isinstance(data, str) else data
        self.send_response(200)
        self.send_header('Content-Type', mime)
        self.send_header('Content-Length', str(len(data)))
        self.send_header('Cache-Control', 'no-store')
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        url = urlsplit(self.path)
        if url.path in ('/', '/qa'):
            return self.send(PAGE, 'text/html; charset=utf-8')
        if url.path in ('/qa/builtin-runtime.js', '/qa/builtin-runtime.css'):
            name = url.path.rsplit('/', 1)[1]
            return self.send((ANDROID / name).read_bytes(), 'text/javascript' if name.endswith('.js') else 'text/css')
        if url.path == '/water-sort/index.html' and parse_qs(url.query).get('android') == ['1']:
            return self.send((PACK / 'index.html').read_text(encoding='utf-8').replace('<head>', '<head>' + SDK), 'text/html; charset=utf-8')
        return super().do_GET()

if __name__ == '__main__':
    print('Pour Joy preview: http://127.0.0.1:1487/qa', flush=True)
    ThreadingHTTPServer(('127.0.0.1', 1487), Handler).serve_forever()
