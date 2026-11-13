/* 悬山 · nine readable human silhouettes, flat ink tones, shared small-surface pose cache. */
(() => {
  'use strict'
  const F = globalThis.XiabanInkFigures, A = globalThis.XiabanInkArt, TAU = Math.PI * 2
  const STRIKES = {
    thrust:  { ready: -.52, finish: -.02, coil: .04, lunge: .59 },
    cleave:  { ready: -2.38, finish: .78, coil: -.32, lunge: .64 },
    lift:    { ready: .68, finish: -1.35, coil: .13, lunge: .24 },
    shove:   { ready: -1.67, finish: -.10, coil: -.08, lunge: .56 },
    rush:    { ready: -2.89, finish: -.09, coil: .41, lunge: .73 },
    reverse: { ready: .68, finish: -1.09, coil: .21, lunge: .48 },
    sweep:   { ready: -2.92, finish: .31, coil: -.22, lunge: .68 },
    slam:    { ready: -1.85, finish: .78, coil: -.31, lunge: .71 },
  }
  const COSTUMES = {
    umbrella: ['#252c28', '#475345', '#758170', '#b0ac8f', '#e3d7b8'],
    monk: ['#2e3130', '#57584b', '#888270', '#b5ab8f', '#e0cfac'],
    disciple: ['#24292a', '#45504e', '#748278', '#a5ae97', '#ded5ba'],
    rope: ['#272d30', '#46565c', '#6b8185', '#a5aaa0', '#e4d8b7'],
    crossbow: ['#272b30', '#454c56', '#6a7982', '#a3aa9e', '#dcdac3'],
    axeman: ['#292928', '#514642', '#7b6959', '#ad9d7a', '#ded0b1'],
    yanhui: ['#222829', '#3b4c4c', '#647c76', '#aab39b', '#e8d7b7'],
    shizhen: ['#2b2a2b', '#514a49', '#756757', '#ae9b79', '#e7cfa6'],
    yue: ['#202428', '#373c45', '#636b70', '#a6a49a', '#e8d9ba'],
  }
  const YUE_OPEN = ['#202326', '#403a3c', '#786356', '#ad9985', '#f0dab5']
  const TWO_HANDED = new Set(['yue', 'monk', 'axeman', 'shizhen'])

  function pose(e, t, anim, time, q, speed) {
    const heavy = ['axeman', 'shizhen', 'yue'].includes(e.type), bow = e.type === 'crossbow'
    t.lean = heavy ? .09 : .17
    t.crouch = heavy ? 2 : 1
    t.sw = heavy ? .16 : .45
    t.reach = heavy ? .9 : 1
    t.ou = .5; t.of = .2
    t.l1 = .18; t.l2 = -.20; t.b1 = .28; t.b2 = .18
    t.flutter = .07 + Math.sin(time * 2.1 + e.seed) * .05
    if (e.type === 'umbrella') { t.ou = 1.15; t.of = 1.05; t.sw = .77 }
    if (e.type === 'monk') { t.sw = -.41; t.reach = 1.15 }
    if (bow) { t.sw = .04; t.ou = 1.26; t.of = 1.5; t.reach = 1.05 }
    if (e.type === 'yue' && e.phase2) { t.lean = .22; t.sw = -.27; t.l1 = .33; t.l2 = -.36 }
    if (e.state === 'walk' || (speed > 25 && e.state === 'idle')) {
      const a = Math.min(.85, speed / 140), sn = Math.sin(anim.phase), co = Math.cos(anim.phase)
      t.l1 = sn * .86 * a; t.l2 = -sn * .86 * a
      t.b1 = .25 + Math.max(0, co) * a; t.b2 = .25 + Math.max(0, -co) * a
      t.bob = Math.abs(co) * a * 1.4; t.flutter += a * .8
    }
    if (e.state === 'mountTell') {
      const { ready, coil } = STRIKES[e.move] || STRIKES.thrust
      const pull = Math.min(1, q * 2.65)
      t.sw = ready + (1 - pull) * .21
      t.lean = coil - pull * .08; t.crouch = 2.5 + pull * (e.sweep ? 5 : 2.6)
      t.reach = e.move === 'lift' ? .86 : 1.02
      t.l1 = .41; t.l2 = -.55; t.b1 = .51; t.b2 = .23
      if (e.type === 'umbrella') { t.sw = -1.45; t.ou = .85; t.of = 1.21 }
    }
    if (e.state === 'mountAttack') {
      const { finish, lunge } = STRIKES[e.move] || STRIKES.thrust
      t.sw = finish; t.lean = lunge; t.crouch = e.sweep ? 8.5 : 3.5
      t.reach = 1.22; t.l1 = .91; t.l2 = -.62; t.b1 = .74; t.b2 = .1
      t.flutter = .96; anim.snap = true
    }
    if (e.state === 'bowTell') {
      t.sw = -.1; t.crouch = 2 + q * 2; t.lean = -.08; t.reach = 1.19
      t.draw = q; t.l1 = .38; t.l2 = -.42; t.b1 = .42
    }
    if (e.state === 'umbrellaOpen') {
      t.lean = -.22; t.crouch = 3; t.ou = .78; t.of = 1.4
      t.sw = 2.16; t.reach = .77; t.l1 = .39; t.l2 = -.36
    }
    if (e.type === 'rope' && (e.ropeMounted || e.state === 'ropeClimb')) {
      t.air = 1; t.ou = -2.7; t.of = -2.9
      t.sw = e.state === 'ropeDive' ? .77 : e.state === 'ropeTell' ? -2.15 : -1.28
      t.l1 = .94; t.b1 = 1.4; t.l2 = -.38; t.b2 = .53
      t.lean = e.state === 'ropeDive' ? .7 : -.12
      t.flutter = e.state === 'ropeDive' ? 1 : .2
      anim.snap = e.state === 'ropeDive'
    }
    if (e.state === 'recover') {
      t.lean = e.sweep ? .54 : .32; t.crouch = e.sweep ? 6.4 : 3.4
      t.sw = e.move === 'lift' ? -.76 : e.sweep ? .60 : .47
      t.ou = .22; t.of = .61; t.l1 = .54; t.l2 = -.42
    }
    if (['hurt', 'stagger', 'broken', 'dying'].includes(e.state)) {
      const kneel = e.state === 'broken' || e.state === 'dying'
      t.lean = kneel ? .52 : -.38; t.crouch = kneel ? 13 : 5
      t.sw = kneel ? .77 : -2.1; t.ou = 2; t.of = 2.4
      t.l1 = kneel ? 1.25 : .56; t.l2 = kneel ? -.95 : -.32
      t.b1 = kneel ? 1.82 : .6; t.b2 = kneel ? 1.17 : .24; t.head = kneel ? .27 : -.25
    }
    if (e.state === 'roar') {
      const step = Math.sin(anim.phase)
      t.sw = -1.54; t.lean = -.16; t.l1 = step * .57; t.l2 = -step * .57
      t.flutter = .98; t.reach = .83
    }
    return t
  }
  function paint(g,kind,p,x,y,facing,look={},time=0,seed=0){
    const s=F.skeleton(F.KINDS[kind],p),e=look.entity||{},flat=look.color||null
    const colors = kind === 'yue' && e.phase2 ? YUE_OPEN : COSTUMES[kind]
    const C = n => flat || colors[n]
    const [hx,hy]=s.head,[sx,sy]=s.shoulder,[bx,by]=s.hip
    let tip=s.hand
    g.save();g.translate(x,y);g.scale(facing,1);if(look.alpha!==undefined)g.globalAlpha*=look.alpha;g.lineJoin='round';g.lineCap='round'
    const fill=(pts,n=1)=>{g.fillStyle=C(n);g.beginPath();pts.forEach((v,i)=>i?g.lineTo(...v):g.moveTo(...v));g.closePath();g.fill()}
    const line=(pts,n=2,w=.7)=>{g.strokeStyle=C(n);g.lineWidth=w;g.beginPath();pts.forEach((v,i)=>i?g.lineTo(...v):g.moveTo(...v));g.stroke()}
    function arm(a, b, c, w = 4, n = 1) {
      g.fillStyle = C(0)
      A.ribbon(g, [a, b, c], t => w * (2.2 - t * .9))
      g.fillStyle = C(n)
      A.ribbon(g, [a, b, c], t => w * (1.8 - t * .8))
      line([[a[0] - 1, a[1] + 2], [b[0] - 1, b[1] + 1], [c[0] - 1, c[1] + 1]], Math.min(4, n + 1), .7)
      g.fillStyle = C(3); g.beginPath(); g.ellipse(c[0], c[1], 2.35, 2.1, -.4, 0, TAU); g.fill()
    }
    function leg(l) {
      arm(s.hip, l.knee, l.foot, kind === 'shizhen' ? 4 : 3.35, 1)
      fill([[l.foot[0]-3,l.foot[1]-6],[l.foot[0]+2,l.foot[1]-5],[l.foot[0]+7,l.foot[1]],[l.foot[0]+6,l.foot[1]+1],[l.foot[0]-3,l.foot[1]+1]],0)
      line([[l.foot[0]-2,l.foot[1]-5],[l.foot[0]+2,l.foot[1]-3]],3,.7)
    }
    function face(shaved=false){
      g.fillStyle=C(3);g.beginPath();g.ellipse(hx,hy,5.5,6.5,p.head*.2,0,TAU);g.fill()
      fill([[hx+3,hy-2],[hx+7,hy+1],[hx+3,hy+3]],3)
      if(!shaved){fill([[hx-6,hy-1],[hx-5,hy-8],[hx+3,hy-9],[hx+6,hy-4],[hx+1,hy-5],[hx-3,hy+3]],0)}
      g.strokeStyle=flat||'#b8322a';g.lineWidth=.9+(look.eyeGlow||0)*.4;g.beginPath();g.moveTo(hx+1,hy-1);g.lineTo(hx+4,hy-1);g.stroke()
      line([[hx+1,hy+4],[hx+3,hy+4]],1,.4)
    }
    function tunic(long = false, n = 1) {
      const heavy = kind === 'shizhen' || kind === 'axeman' || kind === 'yue'
      const width = heavy ? 11 : 8
      const hem = long ? Math.min(s.front.foot[1], s.back.foot[1]) - 1 : by + 14
      const tail = p.flutter * (long ? 18 : 9)
      const outline = [[sx - width, sy - 3], [sx + width, sy - 1], [bx + 10, by - 2], [bx + 11, by + 4],
        [bx + 13, hem - 2], [bx + 4, hem - 5], [bx + 1, hem], [bx - 8, hem - 3], [bx - 16 - tail, hem + 1], [bx - 11, by]]
      fill(outline, 0)
      fill([[sx - width + 2, sy - 2], [sx + width - 1, sy], [bx + 9, by - 1], [bx + 11, hem - 3], [bx + 2, hem - 6],
        [bx - 3, hem - 1], [bx - 14 - tail, hem - 1], [bx - 10, by]], n)
      fill([[sx - 8, sy - 1], [sx - 2, sy + 2], [bx + 8, by + 1], [bx + 3, by + 4]], 3)
      fill([[sx + 3, sy - 1], [sx + 7, sy], [bx - 5, by + 1], [bx - 8, by + 2]], 2)
      line([[sx + 4, sy], [bx - 7, by + 2]], 0, 1.45)
      for (let j = 0; j < 4; j++) {
        const h = bx - 12 + j * 7 - tail * (1 - j / 4)
        line([[bx - 8 + j * 5, by + 2], [h + 3, (by + hem) * .5], [h, hem - 3 - j % 3]], Math.min(3, n + 1), .48 + j % 2 * .21)
      }
      line([[bx - 11, by + 1], [bx + 11, by]], 0, 3.4)
      if (kind === 'disciple' || kind === 'yanhui' || kind === 'yue' && e.phase2) {
        g.strokeStyle = flat || '#a55c48'; g.lineWidth = 1.2
        g.beginPath(); g.moveTo(bx - 11, by); g.lineTo(bx + 10, by - .4)
        g.quadraticCurveTo(bx + 2 - tail * .32, by + 5, bx - 4 - tail, by + 10); g.stroke()
      }
    }
    function weaponAngle(len, clearance) {
      if (p.sw <= 0) return p.sw
      return Math.min(p.sw, Math.asin(Math.max(-.95, Math.min(.95, (-s.hand[1] - clearance) / len))))
    }
    function blade(len = 39, broad = false) {
      const angle = broad ? weaponAngle(len, 3) : p.sw
      const thickness = broad ? 5.7 : 1.7
      g.save(); g.translate(...s.hand); g.rotate(angle)
      fill([[-20, -1.7], [-20, 2.1], [0, 2.2], [0, -2]], 0)
      for (const x of [-17, -13, -9, -5]) line([[x, -1.8], [x - 1, 1.5]], 2, .7)
      line([[0, -thickness - 1.5], [1, thickness + 1.5]], 0, 2)
      fill([[2,-thickness],[len-13,-thickness * .95],[len,-.4],[len-9,thickness * .76],[2,thickness * .77]],2)
      fill([[2,-thickness * .18],[len-5,-.45],[len-10,thickness * .72],[2,thickness * .72]],3)
      line([[3, thickness * .57], [len - 10, thickness * .63], [len - 1, 0]], 4, .85)
      if (broad) line([[4, -thickness * .73], [len - 14, -thickness * .64]], 1, .92)
      g.restore()
      tip = [s.hand[0] + Math.cos(angle) * len, s.hand[1] + Math.sin(angle) * len]
    }
    function pole(type) {
      const len = type === 'staff' ? 84 : 74
      const angle = weaponAngle(len, type === 'staff' ? 4 : 11)
      g.save();g.translate(...s.hand);g.rotate(angle)
      line([[-25,0],[len-5,0]],0,3.7)
      line([[-23,-.63],[len-6,-.63]],2,1.48)
      for(const xx of [-19,-14,8,48])line([[xx,-2],[xx,2]],3,1)
      if(type==='axe'){
        fill([[len-19,-4],[len-10,-14],[len+6,-20],[len+10,-10],[len+7,4],[len-2,14],[len-17,11]],0)
        fill([[len-12,-6],[len-6,-13],[len+6,-18],[len+8,-9],[len+4,4],[len-4,12],[len-13,9]],2)
        line([[len+5,-17],[len+8,-9],[len+4,4],[len-4,12]],4,1.35)
      }else if(type==='hammer'){
        fill([[len-23,-11],[len-14,-16],[len+6,-14],[len+9,-9],[len+9,10],[len+3,16],[len-17,14],[len-23,8]],0)
        g.fillStyle=flat||'#947b52';g.beginPath();g.roundRect(len-20,-12,26,25,3);g.fill()
        line([[len-18,-11],[len+4,-11],[len+5,10]],3,1.5)
        line([[len-17,-3],[len-4,-3],[len-4,9]],1,1)
      }else{
        line([[len-14,-2],[len,-2],[len,2],[len-14,2]],3,1.2)
        for (const d of [-4, -9]) line([[len+d,-2],[len+d,2]],1,1)
      }
      g.restore()
      tip=[s.hand[0]+Math.cos(angle)*len,s.hand[1]+Math.sin(angle)*len]
    }
    if (TWO_HANDED.has(kind)) {
      const len = kind === 'yue' ? 71 : kind === 'monk' ? 84 : 74
      const angle = weaponAngle(len, kind === 'yue' ? 3 : kind === 'monk' ? 4 : 11)
      s.offHand = [s.hand[0] - Math.cos(angle) * 15, s.hand[1] - Math.sin(angle) * 15]
      s.offElbow = [(s.offShoulder[0] + s.offHand[0]) * .5 - 3, (s.offShoulder[1] + s.offHand[1]) * .5 + 6]
    }
    leg(s.back)
    if (kind === 'yanhui' || kind === 'yue') {
      const wind = p.flutter * 17, length = kind === 'yue' ? 28 : 19
      fill([[sx - 8, sy - 3], [sx + 2, sy + 2], [bx - 10 - wind * .7, by + length],
        [bx - 22 - wind, by + length - 3], [bx - 24 - wind, by + length + 4], [bx - 16, by + 2]], 0)
      line([[sx - 7, sy], [bx - 14 - wind * .6, by + length - 4], [bx - 21 - wind, by + length + 1]], 2, .95)
    }
    leg(s.front)
    tunic(['umbrella', 'yue', 'yanhui'].includes(kind), kind === 'monk' ? 2 : 1)
    arm(s.offShoulder,s.offElbow,s.offHand,kind==='shizhen'?5:3.5,kind==='monk'?3:1)
    if(kind==='monk'){
      face(true);line([[sx-6,sy],[bx+6,by-2]],0,2)
      for(let j=0;j<7;j++){g.fillStyle=C(1);g.beginPath();g.arc(sx-4+Math.sin(j/6*Math.PI)*9,sy+j*2,1.2,0,TAU);g.fill()}
      arm(s.shoulder,s.elbow,s.hand,4,3);pole('staff')
    }else if(kind==='umbrella'){
      face();fill([[hx-16,hy-2],[hx,hy-12],[hx+17,hy-1],[hx+6,hy+1],[hx-12,hy+1]],0)
      line([[hx-12,hy-2],[hx+12,hy-2]],3,.6)
      const open = e.state === 'umbrellaOpen', ux = s.offHand[0] + 17, uy = sy + (open ? -2 : 6)
      const rx = open ? 25 : 17, ry = open ? 34 : 27
      line([s.offHand, [ux, uy]], 0, 2.9)
      const spokes = Array.from({ length: 10 }, (_, i) => {
        const a = TAU * i / 10
        return [ux + Math.cos(a) * rx - Math.sin(a) * 2, uy + Math.sin(a) * ry]
      })
      fill(spokes, 0)
      for (let i = 0; i < spokes.length; i++) {
        const left = spokes[i], right = spokes[(i + 1) % spokes.length]
        fill([[ux + 1, uy - 1], left, right], i % 3 === 1 ? 2 : 1)
        line([[ux + 1, uy - 1], left], i % 2 ? 3 : 2, .75)
        line([left, right], 0, 1.2)
      }
      g.fillStyle=C(3);g.beginPath();g.arc(ux+1,uy-1,2.3,0,TAU);g.fill()
      arm(s.shoulder,s.elbow,s.hand,3.2);blade(33)
    }else if(kind==='crossbow'){
      face();fill([[hx-10,hy-3],[hx-7,hy-10],[hx+6,hy-9],[hx+12,hy-2]],1)
      fill([[sx-7,sy],[sx+7,sy],[bx+10,by],[bx-10,by]],2)
      line([[sx-4,sy+2],[bx+7,by-1]],0,2)
      arm(s.shoulder,s.elbow,s.hand,4)
      const wx=s.hand[0]+9,wy=s.hand[1]
      fill([[wx-17,wy-3],[wx+30,wy-3],[wx+31,wy],[wx-17,wy+1]],2)
      g.strokeStyle=C(1);g.lineWidth=2;g.beginPath();g.moveTo(wx+15,wy-18);g.quadraticCurveTo(wx+27,wy,wx+15,wy+18);g.stroke();line([[wx+15,wy-18],[wx-1,wy],[wx+15,wy+18]],3,.7)
      if(e.state==='bowTell'){line([[wx-4,wy],[wx+35,wy]],0,.8);fill([[wx+35,wy],[wx+29,wy-2],[wx+29,wy+2]],3)}
      tip=[wx+35,wy]
    }else if(kind==='rope'){
      face();fill([[hx-7,hy-5],[hx+6,hy-5],[hx+6,hy-2],[hx-8,hy-2]],2)
      for(let j=0;j<4;j++){g.strokeStyle=C(3);g.lineWidth=.9;g.beginPath();g.ellipse(bx-10,by+2,6+j,8+j*.7,.1,0,TAU);g.stroke()}
      arm(s.shoulder,s.elbow,s.hand,3);blade(30)
    }else if(kind==='axeman'||kind==='shizhen'){
      face(kind==='shizhen')
      if(kind==='axeman'){fill([[hx-4,hy+4],[hx+4,hy+4],[hx+1,hy+12],[hx-5,hy+8]],0);line([[sx-7,sy],[bx+8,by+2]],3,3)}
      else{line([[hx-6,hy-5],[hx+5,hy-5]],1,2.2);fill([[sx-12,sy-1],[sx+12,sy],[sx+13,sy+10],[sx-12,sy+10]],2)}
      arm(s.shoulder,s.elbow,s.hand,5.2,3);pole(kind==='axeman'?'axe':'hammer')
    }else{
      face();g.fillStyle=C(0);g.beginPath();g.ellipse(hx-3,hy-10,3.5,4,-.3,0,TAU);g.fill()
      if(kind==='yue'){
        fill([[hx - 7, hy + 1], [hx - 5, hy + 5], [hx + 2, hy + 6], [hx + 4, hy + 2], [hx + 1, hy + 9], [hx - 5, hy + 11]], 1)
        line([[hx-5,hy-6],[hx-13,hy+8],[hx-15+Math.sin(time*3)*2,hy+20]],0,3)
        fill([[sx-8,sy],[sx-3,sy],[bx+6,by+2],[bx+3,by+16],[bx,by+4]],3)
      }
      if(kind==='yanhui'){
        const a=p.of-1.6,h=s.offHand;fill([[h[0],h[1]],[h[0]+Math.cos(a)*28-Math.sin(a)*2,h[1]+Math.sin(a)*28+Math.cos(a)*2],[h[0]+Math.cos(a)*33,h[1]+Math.sin(a)*33],[h[0]+Math.sin(a)*2,h[1]-Math.cos(a)*2]],3)
      }
      arm(s.shoulder,s.elbow,s.hand,kind==='yue'?4.5:3.2);blade(kind === 'yue' ? 71 : kind === 'yanhui' ? 43 : 33, kind === 'yue')
    }
    g.restore();const map=v=>[x+v[0]*facing,y+v[1]]
    return {head:map(s.head),tip:map(tip),hand:map(s.hand),neck:map(s.neck),hip:map(s.hip)}
  }
  globalThis.XiabanInkMountainFigures={pose,paint}
})()
