(() => {
  'use strict'
  const root=document.getElementById('publicResult'),data=globalThis.XiaobanMbtiShare.parse(location.search)
  const make=(tag,cls,text)=>{const n=document.createElement(tag);if(cls)n.className=cls;if(text!==undefined)n.textContent=text;return n}
  const start=make('a','public-start','我也来测一测');start.href='https://chat.xiangban.xyz/create?kind=game&game=mbti'
  if(!data){root.replaceChildren(make('h1','','结果链接不完整'),make('p','','可以自己做一次测评，看看你的性格画像。'),start);return}
  const report=globalThis.XiaobanMbtiReport.build(data),order=['INTJ','INTP','ENTJ','ENTP','INFJ','INFP','ENFJ','ENFP','ISTJ','ISFJ','ESTJ','ESFJ','ISTP','ISFP','ESTP','ESFP'],index=order.indexOf(data.type)
  root.style.setProperty('--color',report.family.color)
  const hero=make('section','public-hero'),copy=make('div','public-copy'),sprite=make('div','public-sprite');sprite.style.backgroundPosition=`${index%4*100/3}% ${Math.floor(index/4)*100/3}%`;sprite.setAttribute('role','img');sprite.setAttribute('aria-label',report.profile.role)
  copy.append(make('p','public-group',report.profile.family+' · '+report.family.name),make('h1','',data.type),make('h2','',report.profile.role),make('p','',report.profile.title));hero.append(copy,sprite)
  const section=(number,title)=>{const node=make('section','public-section'),heading=make('h3');heading.append(make('span','public-number',number),make('span','',title));node.append(heading);return node}
  const list=items=>{const node=make('ul','public-list');items.forEach(text=>node.append(make('li','',text)));return node}
  const traits=section('01','四维性格倾向')
  report.dimensions.forEach(d=>{const row=make('div','public-dimension'),labels=make('div','public-labels'),bar=make('div','public-track'),fill=make('i');labels.append(make('span','',`${d.firstLabel} ${d.percent}%`),make('span','',`${d.secondLabel} ${100-d.percent}%`));fill.style.width=d.strength+'%';fill.style.marginLeft=d.percent<50?(100-d.strength)+'%':'0';bar.append(fill);row.append(labels,bar,make('p','public-band',d.label+' · 已答 '+d.total+'/12'),make('p','public-copy-text',d.description));traits.append(row)})
  const strengths=section('02','你的力量与另一面');strengths.append(make('h4','','自然流露的力量'),list(report.profile.strengths),make('h4','','值得照看的另一面'),list(report.profile.blindSpots))
  const work=section('03','你的工作方式'),table=make('table','public-table'),head=make('tr'),thead=make('thead'),body=make('tbody');head.append(make('th','','场景'),make('th','','可以尝试的方式'));thead.append(head);table.append(thead)
  report.profile.work.forEach((copy,i)=>{const row=make('tr');row.append(make('td','',['进入任务','协作推进','节奏调整'][i]),make('td','',copy));body.append(row)});table.append(body);work.append(table)
  const relationships=section('04','你的人际关系');relationships.append(make('p','public-copy-text',report.profile.relationship))
  const growth=section('05','把洞察带进生活');growth.append(list(report.actions))
  root.replaceChildren(make('p','public-brand','小伴 · 自我观察'),hero,make('p','public-summary',report.profile.description),make('p','public-date','测试于 '+globalThis.XiaobanMbtiShare.formattedDate(data.completedAt)+' · 已答 '+report.answered+'/48 题'),traits,strengths,work,relationships,growth,start)
})()
