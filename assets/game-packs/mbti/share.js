/* Offline artwork and public, explicitly selected result snapshot. */
(() => {
  'use strict'
  const origin = 'https://chat.xiangban.xyz'
  const order = ['INTJ','INTP','ENTJ','ENTP','INFJ','INFP','ENFJ','ENFP','ISTJ','ISFJ','ESTJ','ESFJ','ISTP','ISFP','ESTP','ESFP']
  const mottos = {INTJ:'让远见有路径，让想法有回声。',INTP:'追问多一层，世界就多一种可能。',ENTJ:'看清方向，也让同行的人看见。',ENTP:'把好奇变成问题，把问题变成机会。',INFJ:'看见细微，也不忘照亮自己。',INFP:'把珍惜的微光，留在具体的生活里。',ENFJ:'把人与人之间，连成一座桥。',ENFP:'带着好奇出发，让灵感找到落点。',ISTJ:'把每一件小事，做成可靠的支撑。',ISFJ:'那些被记住的细节，都有温度。',ESTJ:'把目标讲清楚，把约定做踏实。',ESFJ:'让每个到来的人，都有自己的位置。',ISTP:'动手试一试，答案就近了一点。',ISFP:'把真实的感受，画成自己的颜色。',ESTP:'把握此刻，也为下一步留好空间。',ESFP:'让今天值得记住，让热情有所回应。'}
  const validType = type => /^[EI][SN][TF][JP]$/.test(type)
  function snapshot(result) {
    if (!result || !validType(result.type) || !Array.isArray(result.percentages) || result.percentages.length !== 4 ||
      !result.percentages.every(n => Number.isInteger(n) && n >= 0 && n <= 100) ||
      !Array.isArray(result.totals) || result.totals.length !== 4 || !result.totals.every(n => Number.isInteger(n) && n >= 1 && n <= 12)) throw new Error('结果不完整')
    const date = new Date(result.completedAt)
    if (!Number.isFinite(date.getTime()) || date.getTime() < 0 || date.getTime() > 4102444800000) throw new Error('测试日期无效')
    const expected = ['EI','SN','TF','JP'].map((d, i) => d[result.percentages[i] >= 50 ? 0 : 1]).join('')
    if (result.type !== expected) throw new Error('结果数据不一致')
    return { type: result.type, percentages: [...result.percentages], totals: [...result.totals], completedAt: date.toISOString() }
  }
  function link(result) {
    const data = snapshot(result)
    const params = new URLSearchParams({ t: data.type, p: data.percentages.join(','), n: data.totals.join(','), d: data.completedAt })
    return `${origin}/builtin-games/mbti/share.html?${params}`
  }
  function parse(search) {
    try {
      const p = new URLSearchParams(search), integerList = value => value && /^\d{1,3}(,\d{1,3}){3}$/.test(value) ? value.split(',').map(Number) : null
      if ([...p.keys()].length !== 4 || !['t','p','n','d'].every(key => p.getAll(key).length === 1)) return null
      const type = p.get('t'), percentages = integerList(p.get('p')), totals = integerList(p.get('n')), completedAt = p.get('d')
      if (!completedAt || !/^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d\.\d{3}Z$/.test(completedAt)) return null
      const result = snapshot({ type, percentages, totals, completedAt })
      return result.completedAt === completedAt ? result : null
    } catch { return null }
  }
  const pendingAssets = new Map()
  let nextAssetId = 0
  const hostNonce = new URLSearchParams(location.search).get('hostNonce')
  if (globalThis.origin === 'null' && hostNonce && window.parent !== window) window.addEventListener('message', event => {
    const d=event.data
    if (event.source!==window.parent || event.origin!==new URL(location.href).origin || !d ||
      d.channel!=='xiaban.game-host' || d.protocol!==1 || d.nonce!==hostNonce || d.type!=='mbti.asset' || !Number.isSafeInteger(d.id)) return
    const waiting=pendingAssets.get(d.id); if(!waiting)return
    pendingAssets.delete(d.id);clearTimeout(waiting.timer)
    if(!(d.blob instanceof Blob) || d.blob.type!=='image/png' || d.blob.size>3*1024*1024) waiting.reject(new Error('图片素材不可用'))
    else waiting.resolve(d.blob)
  })
  function hostAsset(key) {
    return new Promise((resolve,reject)=>{
      const id=++nextAssetId,timer=setTimeout(()=>{pendingAssets.delete(id);reject(new Error('图片素材加载超时'))},10000)
      pendingAssets.set(id,{resolve,reject,timer})
      window.parent.postMessage({channel:'xiaban.game-pack',protocol:1,nonce:hostNonce,type:'mbti.asset',id,key},'*')
    })
  }
  async function image(path, key) {
    const img = new Image()
    let objectUrl
    if(globalThis.origin==='null' && hostNonce && window.parent!==window) {
      objectUrl=URL.createObjectURL(await hostAsset(key)); img.src=objectUrl
    } else img.src=path
    try { await img.decode() } finally { if(objectUrl)URL.revokeObjectURL(objectUrl) }
    return img
  }
  function formattedDate(value) {
    const d = new Date(value), pad = x => String(x).padStart(2,'0')
    return `${d.getFullYear()}.${pad(d.getMonth()+1)}.${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
  }
  async function poster(result) {
    const data = snapshot(result), report = globalThis.XiaobanMbtiReport.build(data)
    if (!report) throw new Error('结果不可用')
    const [art, qr] = await Promise.all([data.type === 'INFJ' ? image('./assets/observer-jade.png','observer') : image('./assets/characters.png','characters'), image('./assets/test-qr.png','qr')])
    const canvas = document.createElement('canvas'); canvas.width = 1080; canvas.height = 1440
    const c = canvas.getContext('2d'), color = report.family.color, index = order.indexOf(data.type)
    c.fillStyle = '#fffdf5'; c.fillRect(0,0,1080,1440); c.fillStyle = color; c.fillRect(0,0,1080,16)
    const text=(value,x,y,size,fill='#23332c',font='sans-serif')=>{c.fillStyle=fill;c.font=`${size}px ${font}`;c.fillText(value,x,y)}
    text('小伴 · 自我观察',64,83,27,'#62746a');text(report.profile.family+' / '+report.family.name,840,83,26,'#62746a')
    text(data.type,64,256,145,color,'Georgia,serif');text(report.profile.role+' · '+report.profile.title,70,330,42,'#23332c','serif')
    const keywords=report.dimensions.map(d=>d.pole===d.first?d.firstLabel:d.secondLabel).join(' · ');text(keywords,72,380,27,'#62746a')
    if (data.type === 'INFJ') c.drawImage(art,79,400,922,615)
    else { const cw=art.naturalWidth/4,ch=art.naturalHeight/4;c.drawImage(art,index%4*cw,Math.floor(index/4)*ch,cw,ch,265,405,550,585) }
    c.textAlign='center';text(mottos[data.type],540,1050,37,'#304c3b','serif');c.textAlign='left'
    report.dimensions.forEach((d,i)=>{const x=70+i*240;text(`${d.pole}  ${d.strength}%`,x,1133,29,'#526259');c.fillStyle='#e1e9e3';c.fillRect(x,1155,200,8);c.fillStyle=color;c.fillRect(x,1155,200*d.strength/100,8)})
    c.strokeStyle='#dbe5d9';c.beginPath();c.moveTo(64,1208);c.lineTo(1016,1208);c.stroke()
    text('测试于 '+formattedDate(data.completedAt),70,1275,29);text(`已答 ${report.answered} / 48 题`,70,1320,24,'#62746a');text('小伴 · 从日常选择认识自己',70,1380,23,'#62746a')
    c.imageSmoothingEnabled=false;c.drawImage(qr,840,1220,176,176);c.imageSmoothingEnabled=true;c.textAlign='center';text('扫码测一测',928,1420,22);c.textAlign='left'
    return canvas.toDataURL('image/png')
  }
  globalThis.XiaobanMbtiShare = { snapshot, link, parse, poster, formattedDate }
})()
