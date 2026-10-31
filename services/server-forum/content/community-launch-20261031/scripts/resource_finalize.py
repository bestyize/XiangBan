"""Build ZIP/PNG/catalog into a separate output directory. Never overwrite a released object."""
from pathlib import Path
import json,hashlib,zipfile,textwrap,re,subprocess
from PIL import Image,ImageDraw,ImageFont
from pack_build_common import *
CAT=json.loads((OUT/'draft-catalog.json').read_text())
FONT=ARGS.font
if not FONT:
 for candidate in ['/System/Library/Fonts/STHeiti Light.ttc','/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc',str(Path(os.environ.get('WINDIR','C:/Windows'))/'Fonts'/'msyh.ttc')]:
  if Path(candidate).is_file():FONT=candidate;break
if not FONT:raise SystemExit('A CJK font is required: pass --font or set XB_CJK_FONT')
def font(n):return ImageFont.truetype(FONT,n)
def wrap(s,width):
 lines=[];line=''
 for c in s:
  if c=='\n' or ImageDraw.Draw(Image.new('RGB',(1,1))).textlength(line+c,font=font(25))>width:
   lines.append(line);line='' if c=='\n' else c
  else:line+=c
 if line:lines.append(line)
 return lines
DATA=OUT/'data';DATA.mkdir(exist_ok=True)
report={'count':0,'kinds':{},'zip_checks':[],'cover_sizes':[]}
for index,item in enumerate(CAT):
 p=Path(item['source_path']);kind=item['kind'];slug=item['slug'];m=json.loads((p/'manifest.json').read_text())
 if m.get('manualOnly'):
  prompt=(p/'prompt.md').read_text();data=json.loads((p/(kind+'.json')).read_text())
  m.update({'prompt':prompt,'canDo':[item['summary'],'提供可填写工作页、结构化空模板及具体示例'],'cannotDo':['不自动创建或执行任务、提醒和目标','不自动访问文件、联网、安装或发布','不持久保存填写内容，需由本人保存'],'permissions':['无需系统权限；由你手动打开和填写文件'],'runLocation':'下载解压后，在本地文本编辑器或你选择的对话工具中手动使用'})
  put(p,'manifest.json',m)
 item['manifest']=m
 assert m['schemaVersion']==1 and m['kind']==kind and m['id']==slug and m['version']=='1.0.0'
 assert re.fullmatch('[a-z0-9][a-z0-9-]{1,39}',slug)
 assert len(m['name'])<=24
 if kind in ['widget','game']:assert len(m['description'])<=80
 files=sorted(f for f in p.rglob('*') if f.is_file())
 if kind in ['wallpaper','widget']:assert {f.name for f in files}=={'manifest.json','index.html'}
 if kind=='effect':assert {f.name for f in files}=={'manifest.json','effect.json'}
 assert all(not f.is_symlink() for f in files)
 for f in files:
  content=f.read_bytes();assert len(content)<4*1024*1024;content.decode('utf8')
  if f.suffix=='.json':json.loads(content)
  if f.suffix in ['.html','.js','.css']:
   text=content.decode();assert not re.search(r'https?://|localStorage|indexedDB|document.cookie|fetch\s*\(',text),f
  if kind=='widget' and f.suffix=='.html':assert '<script' not in f.read_text().lower()
  if f.suffix=='.js':subprocess.run(['node','--check',str(f)],check=True,capture_output=True)
 filekey=f'packs/{kind}/{slug}-1.0.0.zip';coverkey=f'covers/{slug}-1.0.0.png'
 zpath=DATA/filekey;zpath.parent.mkdir(parents=True,exist_ok=True)
 with zipfile.ZipFile(zpath,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=9)as z:
  for f in files:
   info=zipfile.ZipInfo(f.relative_to(p).as_posix(),date_time=(2026,10,31,0,0,0));info.compress_type=zipfile.ZIP_DEFLATED;info.external_attr=0o100644<<16;z.writestr(info,f.read_bytes())
 raw=zpath.read_bytes();sha=hashlib.sha256(raw).hexdigest()
 with zipfile.ZipFile(zpath)as z:
  assert z.testzip() is None;assert len(z.namelist())==len(set(n.lower()for n in z.namelist()));assert len(z.namelist())<=110
  assert all(not n.startswith('/') and '..'not in n.split('/')and '\\'not in n and ':'not in n for n in z.namelist());assert json.loads(z.read('manifest.json'))==m
 limit={'effect':1,'routine':1,'reminder':1,'task':1,'goal':1,'expert':1,'widget':8,'game':8,'skill':8,'workflow':8,'wallpaper':32}[kind]*1024*1024
 assert len(raw)<limit
 cover=DATA/coverkey;cover.parent.mkdir(exist_ok=True)
 bg,accent=PALETTE[kind];im=Image.new('RGB',(1200,750),bg);d=ImageDraw.Draw(im)
 shot=OUT/'screenshots'/f'{slug}.png'
 if kind in ['wallpaper','effect']:
  assert shot.exists(),shot
  im=Image.open(shot).convert('RGB');im=im.resize((1200,750));d=ImageDraw.Draw(im)
 elif kind in ['widget','game']:
  assert shot.exists(),shot
  ss=Image.open(shot).convert('RGBA');ss.thumbnail((520,650));x=650+(520-ss.width)//2;y=(750-ss.height)//2
  if kind=='widget':
   d.rounded_rectangle((610,55,1160,695),radius=32,fill='#dfE4d5');im.paste(ss,(x,y),ss)
  else:im.paste(ss,(x,y),ss)
  d.text((65,85),LABEL[kind],font=font(24),fill=accent)
  lines=textwrap.wrap(item['title'],max(7,(len(item['title'])+1)//2) if len(item['title'])>10 else 10);yy=180
  for line in lines:d.text((60,yy),line,font=font(52),fill='#ecf1e5'if kind=='game'else'#243e39');yy+=75
  for i,line in enumerate(wrap(item['summary'],480)):d.text((65,yy+35+i*39),line,font=font(25),fill='#afc4d5'if kind=='game'else'#516d64')
  d.text((65,625),'实际页面渲染 / 离线原创',font=font(20),fill=accent)
 else:
  d.text((64,65),LABEL[kind]+'  /  MANUAL',font=font(24),fill=accent)
  for i,line in enumerate(textwrap.wrap(item['title'],max(7,(len(item['title'])+1)//2) if len(item['title'])>10 else 10)):d.text((60,165+i*75),line,font=font(51),fill='#f3eee4')
  yy=340
  for line in wrap(item['summary'],470):d.text((64,yy),line,font=font(25),fill='#ccd0c7');yy+=39
  d.rounded_rectangle((65,600,485,655),radius=15,outline=accent,width=2);d.text((85,613),'工作页 · JSON · 示例 · 提示词',font=font(21),fill=accent)
  d.rounded_rectangle((610,80,1130,665),radius=20,fill='#f3efdf');d.text((650,118),'从一步开始，留下真实记录',font=font(24),fill=bg)
  steps=json.loads((p/(kind+'.json')).read_text())['steps']
  y=185
  for i,step in enumerate(steps):
   d.rounded_rectangle((650,y+5,677,y+32),radius=5,outline=bg,width=2)
   # Cover quotes actual template instructions, and explicitly labels this as a document.
   words=wrap(step['action'],385)
   for j,line in enumerate(words[:2]):d.text((695,y+j*30),line,font=font(21),fill=bg)
   y+=87
  d.line((650,615,1090,615),fill='#d0cfbc',width=1);d.text((650,631),'可下载手动模板 · 不自动执行',font=font(19),fill='#677363')
 im.save(cover,optimize=True)
 item.update({'file_key':filekey,'sha256':sha,'size':len(raw),'cover_key':coverkey,'zip_path':str(zpath),'local_zip':str(zpath),'cover_path':str(cover),'local_cover':str(cover),'cover_sha256':hashlib.sha256(cover.read_bytes()).hexdigest(),'cover_size':cover.stat().st_size,'cover_method':'desktop-effect-engine screenshot'if kind=='effect'else'actual HTML screenshot'if kind in ['wallpaper','widget','game']else'original code-native template illustration'})
 report['count']+=1;report['kinds'][kind]=report['kinds'].get(kind,0)+1;report['zip_checks'].append({'slug':slug,'files':len(files),'zip_size':len(raw),'sha256':sha,'rootManifest':True,'utf8':True,'safePaths':True});report['cover_sizes'].append({'slug':slug,'bytes':cover.stat().st_size,'format':'PNG','width':1200,'height':750})
assert len(CAT)==55 and set(report['kinds'].values())=={5}
put(OUT,'catalog.json',CAT);put(OUT,'structure-validation.json',report)
print('validated and packaged',len(CAT),'objects: 55 ZIP + 55 PNG; bytes',sum(x['size']+x['cover_size']for x in CAT))
