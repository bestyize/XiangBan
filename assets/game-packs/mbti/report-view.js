(() => {
  'use strict'
  const order=['INTJ','INTP','ENTJ','ENTP','INFJ','INFP','ENFJ','ENFP','ISTJ','ISFJ','ESTJ','ESFJ','ISTP','ISFP','ESTP','ESFP']
  const el=(tag,className,text)=>{const n=document.createElement(tag);if(className)n.className=className;if(text!==undefined)n.textContent=text;return n}
  function section(id,index,title){const n=el('section','report-section');n.id='report-'+id;const h=el('h3','report-section-title');h.append(el('span','report-index',index),el('span','',title));n.append(h);return n}
  function list(items){const n=el('ul','report-list');items.forEach(text=>n.append(el('li','',text)));return n}
  function context(row){
    const n=el('article','report-context');n.append(el('h4','',row.title+' · '+row.headline),el('p','',row.observation),el('p','report-action',row.action))
    const detail=el('details','report-evidence');detail.append(el('summary','',`来自你的 ${row.evidence.length} 个选择`))
    row.evidence.forEach(answer=>{const p=el('p');p.append(el('span','',answer.prompt),el('span','report-answer',answer.answer));detail.append(p)})
    n.append(detail);return n
  }
  function render(report,root){
    document.documentElement.style.setProperty('--report-color',report.family.color)
    document.documentElement.style.setProperty('--accent',report.family.color)
    document.documentElement.style.setProperty('--accent-soft',`color-mix(in srgb,${report.family.color} 9%,white)`)
    root.replaceChildren()
    const hero=el('header','report-hero'),identity=el('div','report-identity')
    identity.append(el('p','eyebrow',report.profile.family+' · '+report.family.name))
    const type=el('h2','report-type',report.type);type.id='resultType';type.tabIndex=-1
    identity.append(type,el('p','report-role',report.profile.role),el('p','report-title',report.profile.title))
    const sprite=el('div','report-character');const pos=order.indexOf(report.type)
    sprite.style.backgroundPosition=`${pos%4*100/3}% ${Math.floor(pos/4)*100/3}%`;sprite.setAttribute('role','img');sprite.setAttribute('aria-label',report.type+' '+report.profile.role)
    hero.append(identity,sprite)
    const date=el('div','report-date');date.append(el('span','',`已答 ${report.answered} / 48 题`),el('time','',new Date(report.completedAt).toLocaleString('zh-CN',{year:'numeric',month:'2-digit',day:'2-digit',hour:'2-digit',minute:'2-digit',hour12:false})));date.querySelector('time').dateTime=report.completedAt
    root.append(hero,date,el('p','report-intro',report.profile.description))
    const nav=el('nav','report-nav');nav.setAttribute('aria-label','报告章节')
    for(const [id,label] of [['traits','倾向'],['strengths','特质'],['work','工作'],['relationships','关系'],['growth','成长'],['closing','尾声']]){const b=el('button','',label);b.type='button';b.addEventListener('click',()=>{const app=document.getElementById('app'),target=document.getElementById('report-'+id);app.scrollTo({top:app.scrollTop+target.getBoundingClientRect().top-app.getBoundingClientRect().top-nav.offsetHeight-8,behavior:'auto'})});nav.append(b)}
    root.append(nav)
    const traits=section('traits','01','四维性格倾向')
    report.dimensions.forEach(d=>{const n=el('article','report-dimension'),labels=el('div','report-dimension-label');labels.append(el('span','',`${d.firstLabel} ${d.first} ${d.percent}%`),el('span','',`${d.secondLabel} ${d.second} ${100-d.percent}%`));const bar=el('div','report-track'),fill=el('i');fill.style.width=d.strength+'%';fill.style.marginLeft=d.percent<50?(100-d.strength)+'%':'0';bar.append(fill);bar.setAttribute('role','img');bar.setAttribute('aria-label',labels.textContent);n.append(labels,bar,el('div','report-band',`${d.label} · 已答 ${d.total}/12`),el('p','',d.description));traits.append(n)})
    const strengths=section('strengths','02','你的力量与另一面');strengths.append(el('h4','','自然流露的力量'),list(report.profile.strengths),el('h4','','值得照看的另一面'),list(report.profile.blindSpots))
    const work=section('work','03','你的工作方式');const table=el('table','report-table'),head=el('tr');head.append(el('th','','场景'),el('th','','可以尝试的方式'));const thead=el('thead');thead.append(head);table.append(thead);const tbody=el('tbody');report.profile.work.forEach((copy,i)=>{const tr=el('tr');tr.append(el('td','',['进入任务','协作推进','节奏调整'][i]),el('td','',copy));tbody.append(tr)});table.append(tbody);work.append(table)
    report.contexts.filter(c=>c.section==='work').forEach(c=>work.append(context(c)))
    const relationships=section('relationships','04','你的人际关系');relationships.append(el('p','',report.profile.relationship));report.contexts.filter(c=>c.section==='relationships').forEach(c=>relationships.append(context(c)))
    const growth=section('growth','05','把洞察带进生活');report.actions.forEach((copy,i)=>{const n=el('div','report-step');n.append(el('span','report-step-number',String(i+1).padStart(2,'0')),el('p','',copy));growth.append(n)})
    report.contexts.filter(c=>c.section==='life').forEach(c=>growth.append(context(c)))
    const closing=section('closing','06','写给此刻的你');closing.append(el('blockquote','report-closing',report.profile.title),el('p','',report.profile.growth[2]+'把这件小事留给本周，看看完成后的自己有什么不同。'))
    if(!report.hasAnswerSnapshot)closing.append(el('p','report-legacy','这份历史记录保留了类型和四维分数。下次测评会同时留下具体情境分析。'))
    root.append(traits,strengths,work,relationships,growth,closing)
  }
  globalThis.XiaobanMbtiReportView={render}
})()
