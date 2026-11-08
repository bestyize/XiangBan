/* Original report copy and sample values, for design review only. */
(() => {
  const art = 'app-plaza-assets/mbti-observer-jade.png';
  const date = '2026.11.08';
  const time = '21:46';
  const typeOrder=['INTJ','INTP','ENTJ','ENTP','INFJ','INFP','ENFJ','ENFP','ISTJ','ISFJ','ESTJ','ESFJ','ISTP','ISFP','ESTP','ESFP'];
  const mottos={INTJ:'让远见有路径，让想法有回声。',INTP:'追问多一层，世界就多一种可能。',ENTJ:'看清方向，也让同行的人看见。',ENTP:'把好奇变成问题，把问题变成机会。',INFJ:'看见细微，也不忘照亮自己。',INFP:'把珍惜的微光，留在具体的生活里。',ENFJ:'把人与人之间，连成一座桥。',ENFP:'带着好奇出发，让灵感找到落点。',ISTJ:'把每一件小事，做成可靠的支撑。',ISFJ:'那些被记住的细节，都有温度。',ESTJ:'把目标讲清楚，把约定做踏实。',ESFJ:'让每个到来的人，都有自己的位置。',ISTP:'动手试一试，答案就近了一点。',ISFP:'把真实的感受，画成自己的颜色。',ESTP:'把握此刻，也为下一步留好空间。',ESFP:'让今天值得记住，让热情有所回应。'};
  const typeData=s=>globalThis.MbtiTypeData[s.mbtiType]||globalThis.MbtiTypeData.INFJ;
  function persona(type,className){const n=document.createElement('div'),i=typeOrder.indexOf(type);n.className=className+' mt-persona';n.style.backgroundPosition=`${i%4*100/3}% ${Math.floor(i/4)*100/3}%`;n.setAttribute('role','img');n.setAttribute('aria-label',type+' '+globalThis.MbtiTypeData[type].profile.role);return n}
  function themed(html,s,kind){
    const d=typeData(s),p=d.profile,t=document.createElement('template');t.innerHTML=html;const root=t.content.querySelector('.mr');
    root.style.setProperty('--r-accent',d.family.color);root.style.setProperty('--r-deep',`color-mix(in srgb,${d.family.color} 65%,#17291f)`);root.style.setProperty('--r-soft',`color-mix(in srgb,${d.family.color} 7%,white)`);
    t.content.querySelector('.ms-poster')?.setAttribute('aria-label',d.type+' 精简分享海报');
    const set=(selector,text)=>t.content.querySelectorAll(selector).forEach(n=>n.textContent=text);
    set('.mr-code,.ms-code,.ms-link-type',d.type);set('.mr-family,.ms-top>span:last-child',p.family+' · '+d.family.name);
    set('.mr-identity h1',p.role);set('.mr-subtitle',p.title);set('.ms-title,.ms-link-stage h2',p.role+' · '+p.title);
    set('.ms-subtitle',d.dimensions.map(x=>(x.pole===x.first?x.firstLabel:x.secondLabel)).join(' · '));set('.ms-quote,.ms-link-stage p',mottos[d.type]);
    set('.ms-link-card>span',`我的人格倾向是 ${d.type} · ${p.role} · 小伴人格测评`);
    t.content.querySelectorAll('.ms-axis').forEach((n,i)=>{const x=d.dimensions[i];n.querySelector('span').textContent=x.pole+' '+x.strength+'%';n.querySelector('b').style.width=x.strength+'%'});
    if(d.type!=='INFJ')for(const selector of ['.mr-hero-art','.ms-art','.ms-link-stage>img','.ms-link-card>img','.mr-chapter-art'])t.content.querySelectorAll(selector).forEach(n=>n.replaceWith(persona(d.type,n.className|| (selector.includes('link-card')?'ms-link-persona':'ms-stage-persona'))));
    if(kind==='report'){
      set('.mr-lead blockquote',mottos[d.type]);set('.mr-lead>p',p.description);
      const tags=t.content.querySelector('.mr-tags');tags.replaceChildren(...d.dimensions.map(x=>{const n=document.createElement('span');n.textContent=(x.pole===x.first?x.firstLabel:x.secondLabel)+' '+x.pole;return n}));
      t.content.querySelectorAll('.mr-trait').forEach((n,i)=>{const x=d.dimensions[i],first=x.pole===x.first;const labels=n.querySelectorAll('.mr-trait-labels>span');labels[0].textContent=`${first?x.firstLabel:x.secondLabel} ${x.pole} · ${x.strength}%`;labels[1].textContent=`${first?x.secondLabel:x.firstLabel} ${first?x.second:x.first} · ${100-x.strength}%`;n.querySelector('.mr-track').style.setProperty('--v',x.strength+'%');n.querySelector('.mr-trait-caption>span').textContent=x.label;n.querySelector('p').textContent=x.description});
      set('[data-chapter="traits"]>.mr-note',d.dimensions[3].description);
      const lens=s.mbtiLens||'strengths',items=lens==='strengths'?p.strengths:p.blindSpots;
      t.content.querySelectorAll('.mr-insight').forEach((n,i)=>{n.querySelector('h3').textContent=items[i];n.querySelector('p').textContent=lens==='strengths'?p.work[i]:p.growth[i]});
      set('[data-chapter="work"]>.mr-intro',p.work.join(''));
      t.content.querySelectorAll('.mr-table tbody tr').forEach((n,i)=>{if(i>=p.work.length)n.remove();else n.children[1].textContent=p.work[i]});
      set('[data-chapter="relations"]>.mr-intro',p.relationship);
      t.content.querySelectorAll('.mr-plan label').forEach((n,i)=>{const span=n.querySelector('span'),day=span.querySelector('small');span.replaceChildren(day,document.createTextNode(d.actions[i]));});
      set('.mr-closing>blockquote',mottos[d.type]);const closing=t.content.querySelectorAll('.mr-closing>p');closing[0].textContent=p.description;closing[1].textContent=p.growth[2]+'把这件小事留给本周，看看完成后的自己有什么不同。';
    }
    return t.innerHTML;
  }
  const axes = [
    ['I','内倾',68,'E','外倾','能量来源','你可能更习惯在独处中恢复精力。与熟悉的人深入交流，同样可以成为你的能量来源。'],
    ['N','直觉',61,'S','实感','信息偏好','你会先寻找线索之间的联系，再回头核对细节。做重要决定时，补充具体事实会让直觉更有支撑。'],
    ['F','情感',72,'T','思考','决策方式','你更常把人的感受和自己的价值观纳入判断。照顾关系与清楚表达事实，可以同时发生。'],
    ['J','判断',58,'P','知觉','生活节奏','你略偏向先安排再行动，但这次的两侧比较接近。不同任务里，你可能也享受即兴调整。']
  ];
  const logo = '<span class="mr-logo" aria-hidden="true"><i></i><i></i><i></i><i></i></span>';
  const head = (title, back=false) => `<header class="mr-head">${back?'<button class="mr-back" data-a="mbti-report" aria-label="返回报告">‹</button>':logo}<span>${title}</span></header>`;
  const sectionHead = (number,title) => `<div class="mr-section-head"><span>${number}</span><h2>${title}</h2></div>`;
  const traits = () => axes.map(([letter,name,value,other,otherName,axis,desc]) => `<details class="mr-trait"><summary><div class="mr-trait-labels"><span class="win">${name} ${letter} · ${value}%</span><small>${axis}</small><span>${otherName} ${other} · ${100-value}%</span></div><div class="mr-track" style="--v:${value}%"><i></i></div><div class="mr-trait-caption"><span>${value<60?'两侧较接近':'本次回答更偏向'+name}</span><span>查看解读 ＋</span></div></summary><p>${desc}</p></details>`).join('');
  const insightData = {
    strengths:[['01','读懂细微的变化','你可能会注意到谈话里的停顿与情绪变化，愿意给对方完整表达的时间。'],['02','把想法连成一条线','零散信息在你眼中容易形成联系，适合梳理复杂的问题和长线目标。'],['03','有温度，也有原则','你在意事情怎样影响他人，也希望行动与自己的价值观一致。']],
    blindspots:[['01','替别人承担太多','当你不断猜测他人的感受，自己的需要可能被放到最后。先问一句，比反复揣测更轻松。'],['02','在理想里反复打磨','想做得周全，有时会让启动变难。给第一版定一个小而清楚的完成标准。'],['03','把分歧当作关系问题','不同意见不一定意味着不被理解。试着把对事实的讨论，与对彼此的感受分开说。']]
  };
  function report(s){
    const lens=s.mbtiLens||'strengths'; const done=s.mbtiDone||[];
    return `<div class="mr">${head('我的人格报告')}<div class="mr-scroll">
      <section class="mr-hero"><div class="mr-eyebrow"><span>PERSONALITY REPORT</span><span>2026 / 自我观察</span></div><div class="mr-portrait"><div class="mr-identity"><span class="mr-family">NF · 外交家</span><div class="mr-code">INFJ</div><h1>提灯者</h1><p class="mr-subtitle">安静的理想主义者</p></div><img class="mr-hero-art" src="${art}" alt="身着绿色外套的提灯者站在书本与树木间的小径上"></div><div class="mr-tags"><span>留意细节</span><span>重视意义</span><span>温和而坚定</span></div></section>
      <div class="mr-lead"><blockquote>看见没有说出口的情绪，<br>也为自己的内心留一盏灯。</blockquote><p>你倾向于在独处中整理想法，在交流中关注感受，也愿意为有意义的事持续投入。</p><div class="mr-meta"><div><b>48 / 48</b><span>情境题</span></div><div><b>08:26</b><span>答题用时</span></div><div><b>${date}</b><span>测试于 ${time}</span></div></div></div>
      <nav class="mr-nav" aria-label="报告章节">${[['traits','倾向'],['strengths','特质'],['work','工作'],['relations','关系'],['growth','成长'],['about','尾声']].map(([id,label])=>`<button class="${id==='traits'?'active':''}" data-a="mbti-jump:${id}">${label}</button>`).join('')}</nav>
      <section class="mr-section" data-chapter="traits">${sectionHead('01','四维性格倾向')}<p class="mr-intro">从能量、信息、决策和节奏四个角度，看看你习惯怎样与世界相处。</p>${traits()}<p class="mr-note"><strong>你的节奏有弹性：</strong>你愿意做计划，也给临时兴起的小事留了位置。先抓住重要节点，细节可以慢慢调整。</p></section>
      <section class="mr-section" data-chapter="strengths">${sectionHead('02','优势与盲点')}<p class="mr-intro">同一种倾向，在合适的场景里是优势，在压力下也可能变成消耗。</p><div class="mr-tabs" role="tablist"><button role="tab" aria-selected="${lens==='strengths'}" class="${lens==='strengths'?'on':''}" data-a="mbti-lens:strengths">性格优势</button><button role="tab" aria-selected="${lens==='blindspots'}" class="${lens==='blindspots'?'on':''}" data-a="mbti-lens:blindspots">潜在盲点</button></div>${insightData[lens].map(([n,title,copy])=>`<article class="mr-insight"><span>${n}</span><div><h3>${title}</h3><p>${copy}</p></div></article>`).join('')}</section>
      <section class="mr-section" data-chapter="work">${sectionHead('03','你的工作方式')}<img class="mr-chapter-art" src="app-plaza-assets/mbti-work.png" alt="提灯者在明亮的工作桌前整理想法"><p class="mr-intro">你更享受先理解问题，再把想法认真做出来。清楚的目标、独立思考的时间和自主空间，会让你更容易进入状态。</p><table class="mr-table"><thead><tr><th>工作情境</th><th>可以尝试的方式</th></tr></thead><tbody><tr><td>启动任务</td><td>先写下“为什么做”，再定一个当天能完成的小版本。</td></tr><tr><td>团队讨论</td><td>提前看议题，留出独立思考时间；会后用文字补充观点。</td></tr><tr><td>接受反馈</td><td>请对方给出具体例子，分清作品需要调整的部分。</td></tr><tr><td>安排节奏</td><td>给深度工作留整块时间，也给临时变化留一点余量。</td></tr></tbody></table><div class="mr-callout">本周小实验：选一件拖延的事，只做一个 30 分钟版本。先让它存在，再让它变好。</div></section>
      <section class="mr-section" data-chapter="relations">${sectionHead('04','你的人际关系')}<img class="mr-chapter-art" src="app-plaza-assets/mbti-relations.png" alt="提灯者与朋友在茶桌边认真交谈"><p class="mr-intro">你可能愿意认真听别人说话，也期待交流有一定深度。把需要说清楚，会比等待对方读懂暗示更容易靠近彼此。</p><div class="mr-dialogue"><div class="bubble">我想认真回应你，能给我一点时间整理想法吗？</div><div class="bubble">可以。你准备好时，我们再继续聊。</div><small>一句具体的表达，让彼此少一点猜测</small></div>${[['和外向的人相处','提前约好一起活动的时间，也留出各自充电的空隙。用一句“我很期待见你，周日想留给自己”，把两份需要都说清楚。'],['面对不同的判断方式','先说明你在意的影响，再讨论可核对的事实。感受与逻辑都可以进入同一场对话。'],['当冲突让你想退开','把“我不想谈”换成“我需要暂停十分钟，之后会回来”。暂停可以有一个明确的归期。']].map(([t,p])=>`<details class="mr-situation"><summary>${t}</summary><p>${p}</p></details>`).join('')}<p class="mr-note">让你自在的关系，往往允许认真交流，也允许安静共处。把期待说清楚，把空间留出来，你会更容易靠近彼此。</p></section>
      <section class="mr-section" data-chapter="growth">${sectionHead('05','把洞察带进生活')}<p class="mr-intro">不需要一次改变很多。留一周时间，试试下面三件小事。</p><div class="mr-plan">${[['第 1 天','写下一个真实需要','用“我需要……”开头，不急着解释或道歉。'],['第 3 天','给不完美留一个出口','交出一个足够清楚的初稿，观察现实中的反馈。'],['第 7 天','回看一次能量变化','记下什么让你舒展，什么需要重新划定边界。']].map(([day,t,p],i)=>`<label><input type="checkbox" data-mbti-check="${i}" ${done.includes(i)?'checked':''}><span><small>${day}</small>${t}<p>${p}</p></span></label>`).join('')}</div><div class="mr-progress-note">已尝试 <span data-mbti-count>${done.length}</span> / 3 项 · 按自己的节奏来</div></section>
      <section class="mr-section mr-closing" data-chapter="about">${sectionHead('06','写给此刻的你')}<p>你习惯在开口之前多想一层，也愿意把一件事放进更长的时间里看。很多时候，你的安静里装着观察、判断，以及还没找到合适时机说出口的在意。</p><p>当这份细腻用在值得投入的人和事上，你会更有力量。让想法有机会落地，让需要有机会被听见，也为独处留一个固定的位置。</p><blockquote>把理解别人的耐心，<br>也分一点给自己。</blockquote><ul class="mr-reading-list"><li><b>关系里</b><br>先表达需要，再期待被理解。</li><li><b>工作中</b><br>为初稿留空间，为专注留时间。</li><li><b>生活里</b><br>把让你舒展的小事，认真放进日程。</li></ul><button class="mr-atlas-link" data-a="mbti-share">把这份认识，留成一张卡 <span>↗</span></button><button class="mr-atlas-link" data-a="mbti-atlas">认识其他人格角色 <span>→</span></button></section>
    </div><footer class="mr-bottom"><span>测试于 ${date}</span><button data-a="mbti-share">分享我的结果 ↗</button></footer></div>`;
  }
  function poster(s){return `<section class="ms-poster ${s.mbtiPaper==='mint'?'mint':''}" aria-label="INFJ 精简分享海报"><div class="ms-top"><span>小伴 · 自我观察</span><span>NF / 外交家</span></div><div class="ms-id"><div class="ms-code">INFJ</div><div class="ms-title">提灯者 · 安静的理想主义者</div><div class="ms-subtitle">温和有边界，安静有方向</div></div><img class="ms-art" src="${art}" alt="INFJ 提灯者原创纸艺插画"><p class="ms-quote">看见细微，也不忘照亮自己。</p>${s.mbtiScores===false?'<div style="flex:1"></div>':`<div class="ms-axes">${axes.map(([l,n,v])=>`<div class="ms-axis"><span>${l}<em style="font-style:normal">${v}%</em></span><i><b style="width:${v}%"></b></i></div>`).join('')}</div>`}<footer class="ms-foot"><div><strong>测试于 ${date}</strong>${time} · 48 道情境题</div><div class="ms-qr"><img src="app-plaza-assets/mbti-test-qr.png" alt="扫码打开小伴 MBTI 测评"><span>扫码测一测</span></div></footer></section>`}
  function share(s){
    const link=s.mbtiShareKind==='link';
    return `<div class="mr ms-share-flow">${head('分享我的结果',true)}<div class="ms-scroll">
      <div class="ms-kind" role="tablist" aria-label="分享方式"><button role="tab" aria-selected="${!link}" class="${!link?'on':''}" data-a="mbti-share-kind:image">图片海报</button><button role="tab" aria-selected="${link}" class="${link?'on':''}" data-a="mbti-share-kind:link">结果链接</button></div>
      ${link?`<div class="ms-link-stage"><span class="ms-link-kicker">分享一个关于自己的小发现</span><div class="ms-link-type">INFJ</div><h2>提灯者 · 安静的理想主义者</h2><img src="${art}" alt="提灯者人格角色"><p>看见细微，也不忘照亮自己。</p></div>
        <h3 class="ms-link-label">朋友圈链接卡片</h3><div class="ms-link-card"><img src="${art}" alt="链接封面"><span>我的人格倾向是 INFJ<br>提灯者 · 小伴人格测评</span></div>
        <div class="ms-link-info"><div><span>测试日期</span><span>${date} ${time}</span></div><div><span>朋友打开后</span><span>精简结果、角色与四维倾向</span></div></div>
        <p class="ms-privacy">朋友会看到你的角色、性格倾向和测试日期。</p>`:
        `<p class="ms-intro">分享一张卡片，留住此刻的自己。</p>${poster(s)}<div class="ms-controls"><div><span>卡片纸色</span><span class="ms-themes"><button class="${s.mbtiPaper!=='mint'?'on':''}" data-a="mbti-paper:ivory">纸白</button><button class="${s.mbtiPaper==='mint'?'on':''}" data-a="mbti-paper:mint">主题色</button></span></div><div><label><input type="checkbox" data-mbti-scores ${s.mbtiScores===false?'':'checked'}> 显示四维倾向</label><span>测试日期始终保留</span></div><p class="ms-privacy">把喜欢的这一版，留给自己或分享给朋友。</p></div>`}
      </div><footer class="ms-share-footer"><div><button data-a="mbti-moments">分享到朋友圈</button><button data-a="${link?'mbti-copy-link':'mbti-save'}">${link?'复制链接':'保存图片'}</button></div><p>会打开微信，由你确认发布</p></footer></div>`;
  }
  const groups=[['NT · 分析家','#8050aa',[['INTJ','构想师'],['INTP','求知者'],['ENTJ','领航者'],['ENTP','破题者']]],['NF · 外交家','#168368',[['INFJ','提灯者'],['INFP','拾光者'],['ENFJ','筑桥者'],['ENFP','追风者']]],['SJ · 守护者','#2685ad',[['ISTJ','守序者'],['ISFJ','育苗者'],['ESTJ','统筹者'],['ESFJ','暖场者']]],['SP · 探险家','#c7900b',[['ISTP','造物者'],['ISFP','绘梦者'],['ESTP','跃行者'],['ESFP','舞光者']]]];
  function atlas(){return `<div class="mr">${head('16 种人格角色',true)}<div class="ms-scroll"><p class="ms-intro">四组主题色，十六种独特的姿态。<br>每一位都拥有自己的动作、轮廓与随身物件。</p>${groups.map(([g,c,roles],row)=>`<section class="ma-group" style="--group:${c}"><h2>${g}</h2><div class="ma-grid">${roles.map(([code,name],col)=>`<article class="ma-character"><div class="ma-sprite" role="img" aria-label="${code} ${name}角色" style="background-position:${col*100/3}% ${row*100/3}%"></div><span>${code}</span><small>${name}</small></article>`).join('')}</div></section>`).join('')}</div></div>`}
  async function makePoster(s){
    const d=typeData(s),p=d.profile,colors=d.family.color,code=d.type;
    const img=new Image();img.src=code==='INFJ'?art:'app-plaza-assets/mbti-character-atlas.png';await img.decode();
    const qr=new Image();qr.src="app-plaza-assets/mbti-test-qr.png";await qr.decode();
    const canvas=document.createElement('canvas');canvas.width=1080;canvas.height=1440;const ctx=canvas.getContext('2d');
    ctx.fillStyle='#fffdf5';ctx.fillRect(0,0,1080,1440);if(s.mbtiPaper==='mint'){ctx.fillStyle=colors+'18';ctx.fillRect(0,0,1080,1440)}ctx.fillStyle=colors;ctx.fillRect(0,0,1080,14);
    const text=(value,x,y,size,color='#20392b',family='sans-serif')=>{ctx.fillStyle=color;ctx.font=`400 ${size}px ${family}`;ctx.fillText(value,x,y)};
    text('小伴 · 自我观察',64,82,26,'#67816e');text(p.family+' / '+d.family.name,827,82,24,'#67816e');text(code,64,270,152,colors,'Georgia');text(p.role+' · '+p.title,70,346,42,'#20392b','serif');text(d.dimensions.map(x=>x.pole===x.first?x.firstLabel:x.secondLabel).join(' · '),72,395,28,'#6c8072');
    if(code==='INFJ')ctx.drawImage(img,79,400,922,615);else{const i=typeOrder.indexOf(code),cw=img.width/4,ch=img.height/4;ctx.drawImage(img,i%4*cw,Math.floor(i/4)*ch,cw,ch,253,410,574,612)}ctx.textAlign='center';text(mottos[code],540,1060,38,'#304c3b','serif');ctx.textAlign='left';
    if(s.mbtiScores!==false)d.dimensions.forEach((dim,i)=>{const x=70+i*241;text(`${dim.pole}    ${dim.strength}%`,x,1137,30,'#43624e');ctx.fillStyle='#e4e6e5';ctx.fillRect(x,1160,202,8);ctx.fillStyle=colors;ctx.fillRect(x,1160,202*dim.strength/100,8)});
    ctx.strokeStyle='#dbe5d9';ctx.beginPath();ctx.moveTo(64,1210);ctx.lineTo(1016,1210);ctx.stroke();text(`测试于 ${date}`,70,1280,29,'#324d3b');text(`${time} · 48 道情境题`,70,1323,24,'#6b7c6b');text('小伴 · 认识自己，从日常选择开始',70,1380,23,'#6b7c6b');
    ctx.imageSmoothingEnabled=false;ctx.drawImage(qr,840,1222,176,176);ctx.imageSmoothingEnabled=true;ctx.textAlign='center';text('扫码测一测',928,1424,22,'#324d3b');ctx.textAlign='left';
    const blob=await new Promise(resolve=>canvas.toBlob(resolve,'image/png'));if(!blob)throw new Error('图片生成失败');return blob;
  }
  async function save(s){const blob=await makePoster(s),url=URL.createObjectURL(blob),a=document.createElement('a');a.href=url;a.download='小伴-'+typeData(s).type+'-20261108.png';document.body.append(a);a.click();a.remove();setTimeout(()=>URL.revokeObjectURL(url),30000)}
  function analysis(s){
    const step=s.analysisStep||0,items=[['汇总你的选择','48 个日常情境'],['梳理性格倾向','能量 · 信息 · 决策 · 节奏'],['整理专属报告','把倾向放回真实生活']];
    return `<div class="mr mx-analysis">${head('性格画像')}<div class="mx-body" aria-live="polite"><div class="mx-eyebrow">PERSONALITY IN THE MAKING</div><h1>${step===3?'你的画像，已准备好':'你的选择，正在连成画像'}</h1><p class="mx-caption">从日常的片段，看见更完整的自己。</p>
      <div class="mx-visual" data-step="${step}" aria-hidden="true"><svg viewBox="0 0 300 240"><defs><linearGradient id="mx-gradient"><stop stop-color="#30b68a"/><stop offset="1" stop-color="#11664d"/></linearGradient></defs><circle class="mx-guide" cx="150" cy="120" r="91"/><circle class="mx-guide inner" cx="150" cy="120" r="62"/><path class="mx-cross" d="M150 15v210M45 120h210"/>
        <path class="mx-thread t1" d="M150 29C232 29 232 211 150 211S68 29 150 29Z"/><path class="mx-thread t2" d="M59 120C59 38 241 38 241 120S59 202 59 120Z"/>
        <g class="mx-mark"><path d="M150 75 189 114 150 114 111 114Z" fill="#8050aa"/><path d="M195 120 156 159 156 120 156 81Z" fill="#168368"/><path d="M150 165 111 126 150 126 189 126Z" fill="#2685ad"/><path d="M105 120 144 81 144 120 144 159Z" fill="#d6a223"/></g>
        <circle class="mx-point p1" cx="150" cy="29" r="5"/><circle class="mx-point p2" cx="241" cy="120" r="5"/><circle class="mx-point p3" cx="150" cy="211" r="5"/><circle class="mx-point p4" cx="59" cy="120" r="5"/></svg><span class="mx-axis top">能量</span><span class="mx-axis right">信息</span><span class="mx-axis bottom">决策</span><span class="mx-axis left">节奏</span></div>
      <div class="mx-steps">${items.map(([title,sub],i)=>`<div class="mx-step ${step>i?'done':step===i?'current':'waiting'}"><span class="mx-step-icon">${step>i?'✓':String(i+1).padStart(2,'0')}</span><div><span>${title}</span><small>${sub}</small></div><i></i></div>`).join('')}</div>
      <div class="mx-progress" aria-label="报告整理进度">${items.map((_,i)=>`<i class="${step>=i?'active':''}"></i>`).join('')}</div><p class="mx-foot">${step===3?'为你打开完整报告':'即将呈现你的专属报告'}</p></div></div>`
  }
  globalThis.MbtiDesign={report:s=>themed(report(s),s,'report'),share:s=>themed(share(s),s,'share'),atlas,save,makePoster,analysis};
})();
