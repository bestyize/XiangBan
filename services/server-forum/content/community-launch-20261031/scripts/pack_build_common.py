from pathlib import Path
import json,math,random,hashlib,zipfile,textwrap
import argparse,os
PARSER=argparse.ArgumentParser(description="Build original offline community resource packs")
PARSER.add_argument('--source-dir',type=Path,default=Path(os.environ.get('XB_PACK_SOURCE_DIR',Path(__file__).resolve().parent.parent/'packs')))
PARSER.add_argument('--output-dir',type=Path,default=Path(os.environ.get('XB_PACK_BUILD_DIR','/tmp/community-pack-rebuild')))
PARSER.add_argument('--font',default=os.environ.get('XB_CJK_FONT'))
ARGS=PARSER.parse_args()
ROOT=ARGS.source_dir.resolve();OUT=ARGS.output_dir.resolve()
ROOT.mkdir(parents=True,exist_ok=True);OUT.mkdir(parents=True,exist_ok=True)
CAT=[]
LABEL={'effect':'桌面特效','wallpaper':'动态壁纸','widget':'桌面组件','game':'离线游戏','routine':'日常流程模板','reminder':'提醒清单模板','task':'任务清单模板','goal':'目标计划模板','skill':'技能说明模板','expert':'专家提示词模板','workflow':'工作流模板'}
PALETTE={'effect':('#091b2c','#6de2d0'),'wallpaper':('#152538','#b0d6f1'),'widget':('#f3ecdc','#21645d'),'game':('#17253b','#f7c65f'),'routine':('#1c3b36','#a4dac0'),'reminder':('#40334d','#e7b1eb'),'task':('#23354d','#94c2f7'),'goal':('#42382b','#e8cb8b'),'skill':('#293b43','#98d5d0'),'expert':('#40383a','#efbaa3'),'workflow':('#343b2e','#bfd799')}
def put(p,name,data):
 p.mkdir(parents=True,exist_ok=True)
 (p/name).write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n' if isinstance(data,(dict,list)) else data,encoding='utf8')
def make(kind,slug,title,summary,tags,extra=None):
 p=ROOT/slug;p.mkdir(exist_ok=True)
 m={'schemaVersion':1,'kind':kind,'id':slug,'name':title,'version':'1.0.0','description':summary,'author':'小伴社区原创作品','license':'MIT'}
 if extra:m.update(extra)
 put(p,'manifest.json',m)
 CAT.append({'kind':kind,'slug':slug,'title':title,'summary':summary,'tags':tags,'manifest':m,'source_path':str(p)})
 return p
