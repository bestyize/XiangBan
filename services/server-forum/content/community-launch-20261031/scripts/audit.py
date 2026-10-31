#!/usr/bin/env python3
"""Read public API filters exactly as community pages use them; never mutate data."""
import argparse,concurrent.futures,json,urllib.parse,urllib.request
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--base',default='https://chat.xiangban.xyz');p.add_argument('--output',required=True);args=p.parse_args()
def get(path):
 with urllib.request.urlopen(args.base+'/api/'+path,timeout=30) as r:return json.load(r)
def page(path,**query):return path+'?'+urllib.parse.urlencode(query)
cats=get('categories');cats=cats.get('items',cats.get('categories',[])) if isinstance(cats,dict) else cats
leaves=[]
def visit(c):
 children=c.get('children',[])
 if children:
  for x in children:visit(x)
 elif c.get('parent_id') is not None:leaves.append(c)
for c in cats:visit(c)
checks=[('论坛/'+c['slug'],page('topics',category=c['slug'],home='false',page_size=1)) for c in leaves]
checks += [('创作/'+k,page('packs',kind=k,page_size=1)) for k in ['effect','wallpaper','widget','game','routine','reminder','task','goal','skill','expert','workflow']]
checks += [('学习/'+t,page('topics',category='tech,share-learn',tag=t,home='false',page_size=1)) for t in ['claude-code','codex','提示词','工作流','本地部署','ai-硬件']]
checks += [('资源/开源精选',page('topics',category='share-software',tag='开源',home='false',page_size=1)),('创作/方案','schemes?page_size=1'),('资源/AI工具','tools'),('网页/项目','projects'),('网页/周刊','weekly?page_size=1')]
def countcheck(pair):
 name,path=pair
 try:
  d=get(path);n=d.get('total',len(d.get('items',[])));return dict(name=name,count=n,minimum=5,pass_=n>=5,path=path)
 except Exception as e:return dict(name=name,count=None,pass_=False,path=path,error=str(e))
rows=list(concurrent.futures.ThreadPoolExecutor(max_workers=4).map(countcheck,checks))
for path,label in [('discover/nav','导航'),('discover/docs','资料')]:
 for group in get(path)['groups']:
  n=len(group['items']);rows.append(dict(name=label+'/'+group['name'],count=n,minimum=5,pass_=n>=5,path=path))
for path,label,key in [('discover/repos/v2','首页/开源热榜','ranks'),('discover/tracker/v2','追踪/发布记录','launches'),('discover/tracker/v2','追踪/优惠与试用','promos'),('discover/home/v2','首页/推荐新闻','news'),('discover/home/v2','首页/精选作品','featured')]:
 try:
  d=get(path);value=d.get(key,[]);n=len(value.get('items',[])) if isinstance(value,dict) else len(value);rows.append(dict(name=label,count=n,minimum=5,pass_=n>=5,path=path))
 except Exception as e:rows.append(dict(name=label,count=None,pass_=False,path=path,error=str(e)))
models=get('discover/models/v2')['models']
for group,key in [('metrics',k) for k in ['intelligence','finance_and_accounting','strategy_and_ops','legal','healthcare_and_medical','engineering','economics']]+[('pricing','input'),('pricing','output'),('performance','output_tokens_per_second')]:
 n=sum(isinstance(m.get(group,{}).get(key),(int,float)) and m[group][key]>=0 for m in models)
 rows.append(dict(name='模型榜/'+key,count=n,minimum=5,pass_=n>=5,path='discover/models/v2'))
n=len(get('discover/model-usage')['models']);rows.append(dict(name='模型榜/调用量',count=n,minimum=5,pass_=n>=5,path='discover/model-usage'))
Path(args.output).write_text(json.dumps(rows,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'checks':len(rows),'passed':sum(x['pass_'] for x in rows),'below_minimum':[{k:v for k,v in x.items() if k in ['name','count','error']} for x in rows if not x['pass_']]},ensure_ascii=False))
