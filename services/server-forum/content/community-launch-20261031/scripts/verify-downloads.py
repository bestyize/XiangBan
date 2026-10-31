#!/usr/bin/env python3
"""Verify real public downloads, hashes, covers and byte ranges without storing signed URLs."""
import argparse,concurrent.futures,hashlib,json,urllib.request,urllib.error
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--base',default='https://chat.xiangban.xyz');p.add_argument('--catalog',required=True);p.add_argument('--output',required=True);a=p.parse_args()
catalog=json.loads(Path(a.catalog).read_text())
def read(url,headers=None):
 with urllib.request.urlopen(urllib.request.Request(url,headers=headers or {}),timeout=30) as r:return r.status,r.headers,r.read()
def get(url):return json.loads(read(url)[2])
lookup={}
for kind in sorted({p['kind'] for p in catalog}):
 for x in get(a.base+'/api/packs?kind='+kind+'&page_size=50')['items']:lookup[(kind,x['slug'])]=x

def verify(p):
 item=lookup[(p['kind'],p['slug'])];info=get(a.base+'/api/packs/'+str(item['id'])+'/download');status,headers,data=read(info['url'])
 assert status==200 and len(data)==p['size'] and hashlib.sha256(data).hexdigest()==p['sha256'],p['slug']
 assert info['sha256']==p['sha256']
 status,h,cover=read(item['cover_url']);assert status==200 and len(cover)==p['cover_size'] and hashlib.sha256(cover).hexdigest()==p['cover_sha256']
 assert h.get_content_type()=='image/png' and h.get('X-Content-Type-Options')=='nosniff'
 status,h,part=read(info['url'],{'Range':'bytes=0-15'});assert status==206 and part==data[:16] and h['Content-Range']==f"bytes 0-15/{len(data)}"
 try:read(info['url'].split('?')[0]);raise AssertionError('unsigned download unexpectedly succeeded')
 except urllib.error.HTTPError as e:assert e.code in (401,403),e.code
 return dict(id=item['id'],kind=p['kind'],slug=p['slug'],title=p['title'],url=a.base+'/packs/'+str(item['id']),size=len(data),sha256=p['sha256'],cover_sha256=p['cover_sha256'],download=True,cover=True,range=True,unsigned_rejected=True)
rows=list(concurrent.futures.ThreadPoolExecutor(max_workers=4).map(verify,catalog))
Path(a.output).write_text(json.dumps(rows,ensure_ascii=False,indent=2)+'\n')
print('Verified',len(rows),'public downloads, covers, ranges and unsigned rejection')
