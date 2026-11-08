/* 悬山 · nine readable human silhouettes, flat ink tones, shared small-surface pose cache. */
(() => {
  'use strict'
  const F=globalThis.XiabanInkFigures,TAU=Math.PI*2
  function pose(e,t,anim,time,q,speed){
    const heavy=['axeman','shizhen','yue'].includes(e.type),bow=e.type==='crossbow'
    t.lean=heavy?.12:.18;t.crouch=heavy?3:1;t.sw=heavy?.35:.5;t.reach=1;t.ou=.5;t.of=.2
    if(e.type==='umbrella'){t.ou=1.2;t.of=1.1;t.sw=.8}
    if(e.type==='monk'){t.sw=-.25;t.reach=1.2}
    if(bow){t.sw=0;t.ou=1.3;t.of=1.5;t.reach=1.1}
    if(e.state==='walk'||speed>25&&e.state==='idle'){
      const a=Math.min(.8,speed/160),sn=Math.sin(anim.phase),co=Math.cos(anim.phase)
      t.l1=sn*.8*a;t.l2=-sn*.8*a;t.b1=.25+Math.max(0,co)*a;t.b2=.25+Math.max(0,-co)*a;t.bob=Math.abs(co)*a;t.flutter=a
    }
    if(['mountTell','ropeTell','bowTell'].includes(e.state)){
      t.sw=bow?-.05:e.sweep?-2.8:heavy?-2.4:e.type==='monk'?-.35:-1.9;t.lean=-.1;t.crouch=4+q*3;t.reach=1.15;t.l1=.55;t.l2=-.4
      if(e.type==='umbrella'){t.sw=-.7;t.ou=.8;t.of=1.2}
    }
    if(e.state==='mountAttack'){
      t.sw=e.sweep?1.05:heavy?1.0:e.type==='monk'?0:.5;t.lean=.6;t.crouch=6;t.reach=1.35;t.l1=.8;t.l2=-.65;t.flutter=.8;anim.snap=true
    }
    if(e.state==='umbrellaOpen'){t.lean=-.2;t.crouch=3;t.ou=.5;t.of=1.0;t.sw=2.3;t.reach=.8}
    if(e.type==='rope'&&(e.ropeMounted||e.state==='ropeClimb')){
      t.air=1;t.ou=-2.7;t.of=-2.9;t.sw=e.state==='ropeDive'?.7:-1.3;t.l1=.8;t.b1=1.2;t.l2=-.3;t.b2=.6;t.lean=e.state==='ropeDive'?.6:0;anim.snap=e.state==='ropeDive'
    }
    if(e.state==='recover'){t.lean=.45;t.crouch=6;t.sw=heavy?.35:1.0;t.ou=.2;t.of=.7}
    if(['hurt','stagger','broken'].includes(e.state)){t.lean=-.4;t.crouch=e.state==='broken'?13:6;t.sw=-2.4;t.ou=2;t.of=2.6;t.l1=.6;t.l2=-.3;t.head=-.25}
    if(e.state==='roar'){t.sw=-2;t.lean=-.1;t.ou=-1.5;t.of=-1.8}
    return t
  }
  function paint(g,kind,p,x,y,facing,look={},time=0,seed=0){
    const s=F.skeleton(F.KINDS[kind],p),e=look.entity||{},flat=look.color||null
    const colors=['#242a25','#474d42','#737969','#a9ac97','#ddd9c4'],C=n=>flat||colors[n]
    const [hx,hy]=s.head,[sx,sy]=s.shoulder,[bx,by]=s.hip
    let tip=s.hand
    g.save();g.translate(x,y);g.scale(facing,1);if(look.alpha!==undefined)g.globalAlpha*=look.alpha;g.lineJoin='round';g.lineCap='round'
    const fill=(pts,n=1)=>{g.fillStyle=C(n);g.beginPath();pts.forEach((v,i)=>i?g.lineTo(...v):g.moveTo(...v));g.closePath();g.fill()}
    const line=(pts,n=2,w=.7)=>{g.strokeStyle=C(n);g.lineWidth=w;g.beginPath();pts.forEach((v,i)=>i?g.lineTo(...v):g.moveTo(...v));g.stroke()}
    function arm(a,b,c,w=4,n=1){line([a,b,c],n,w*2);line([[b[0]-1,b[1]+1],[c[0]-1,c[1]+1]],Math.min(4,n+1),.7);g.fillStyle=C(3);g.beginPath();g.arc(c[0],c[1],2.1,0,TAU);g.fill()}
    function leg(l){arm(s.hip,l.knee,l.foot,3.3,1);fill([[l.foot[0]-3,l.foot[1]-6],[l.foot[0]+2,l.foot[1]-5],[l.foot[0]+7,l.foot[1]],[l.foot[0]+6,l.foot[1]+1],[l.foot[0]-3,l.foot[1]+1]],0);line([[l.foot[0]-2,l.foot[1]-5],[l.foot[0]+2,l.foot[1]-3]],3,.7)}
    function face(shaved=false){
      g.fillStyle=C(3);g.beginPath();g.ellipse(hx,hy,5.5,6.5,p.head*.2,0,TAU);g.fill()
      fill([[hx+3,hy-2],[hx+7,hy+1],[hx+3,hy+3]],3)
      if(!shaved){fill([[hx-6,hy-1],[hx-5,hy-8],[hx+3,hy-9],[hx+6,hy-4],[hx+1,hy-5],[hx-3,hy+3]],0)}
      g.strokeStyle=flat||'#b8322a';g.lineWidth=.9+(look.eyeGlow||0)*.4;g.beginPath();g.moveTo(hx+1,hy-1);g.lineTo(hx+4,hy-1);g.stroke()
      line([[hx+1,hy+4],[hx+3,hy+4]],1,.4)
    }
    function tunic(long=false,n=1){
      const hem=long?0:by+15,tail=p.flutter*9
      fill([[sx-8,sy-3],[sx+8,sy-2],[bx+12,by+2],[bx+15,hem],[bx+3,hem-3],[bx-2,hem],[bx-15-tail,hem+2],[bx-11,by]],n)
      fill([[sx-6,sy-2],[sx-1,sy+1],[bx+7,by+2],[bx+2,by+4]],3)
      line([[sx+3,sy-2],[bx-6,by+2]],0,1.4)
      for(let j=0;j<3;j++)line([[bx-7+j*6,by+1],[bx-9+j*9-tail*(1-j/3),hem-2]],Math.min(3,n+1),.7)
      line([[bx-11,by+1],[bx+11,by]],0,3)
    }
    function blade(len=39,broad=false){
      const angle=broad&&p.sw>0?Math.min(p.sw,Math.asin(Math.max(-.95,Math.min(.95,(-s.hand[1]-3)/len)))):p.sw
      g.save();g.translate(...s.hand);g.rotate(angle)
      fill([[-12,-2],[-12,2],[0,2],[0,-2]],0);line([[0,-5],[0,5]],2,1.5)
      fill([[2,-(broad?4:1.4)],[len-9,-(broad?5:1.5)],[len,0],[len-8,broad?4:2],[2,broad?3:1.5]],3);line([[3,-1],[len-4,0]],4,.65)
      g.restore();tip=[s.hand[0]+Math.cos(angle)*len,s.hand[1]+Math.sin(angle)*len]
    }
    function pole(type){
      const len=type==='staff'?82:76
      const angle=type!=='staff'&&p.sw>0?Math.min(p.sw,Math.asin(Math.max(-.95,Math.min(.95,(-s.hand[1]-14)/len)))):p.sw
      g.save();g.translate(...s.hand);g.rotate(angle)
      line([[-22,0],[len-10,0]],1,3);line([[-20,-.7],[len-12,-.7]],3,.7)
      for(const xx of [-15,8,48])line([[xx,-2],[xx,2]],2,1)
      if(type==='axe'){
        fill([[len-18,-5],[len-6,-17],[len+7,-19],[len+9,-7],[len+3,8],[len-10,14],[len-15,4]],2)
        line([[len+7,-19],[len+9,-7],[len+3,8],[len-10,14]],4,1.1)
      }else if(type==='hammer'){
        g.fillStyle=flat||'#927a50';g.beginPath();g.roundRect(len-21,-14,28,27,3);g.fill();line([[len-18,-11],[len+4,-11],[len+4,10]],3,1)
      }else{line([[len-13,-2],[len,-2],[len,2],[len-13,2]],3,1)}
      g.restore();tip=[s.hand[0]+Math.cos(angle)*len,s.hand[1]+Math.sin(angle)*len]
    }
    leg(s.back)
    if(['yanhui','yue'].includes(kind))fill([[sx-7,sy-3],[sx+2,sy+3],[bx-14-p.flutter*13,by+23],[bx-25-p.flutter*17,by+25],[bx-16,by]],0)
    leg(s.front)
    tunic(['umbrella','yue'].includes(kind),kind==='monk'?2:kind==='rope'?0:1)
    arm(s.offShoulder,s.offElbow,s.offHand,kind==='shizhen'?5:3.5,kind==='monk'?3:1)
    if(kind==='monk'){
      face(true);line([[sx-6,sy],[bx+6,by-2]],0,2)
      for(let j=0;j<7;j++){g.fillStyle=C(1);g.beginPath();g.arc(sx-4+Math.sin(j/6*Math.PI)*9,sy+j*2,1.2,0,TAU);g.fill()}
      arm(s.shoulder,s.elbow,s.hand,4,3);pole('staff')
    }else if(kind==='umbrella'){
      face();fill([[hx-16,hy-2],[hx,hy-12],[hx+17,hy-1],[hx+6,hy+1],[hx-12,hy+1]],0)
      line([[hx-12,hy-2],[hx+12,hy-2]],3,.6)
      const open=e.state==='umbrellaOpen',ux=s.offHand[0]+20,uy=sy+4
      line([s.offHand,[ux,uy]],1,2.5)
      g.fillStyle=C(open?2:1);g.strokeStyle=C(0);g.lineWidth=1
      g.beginPath();g.ellipse(ux,uy,open?26:16,open?34:29,-.1,0,TAU);g.fill();g.stroke()
      for(let j=0;j<8;j++){const a=j/8*TAU;line([[ux,uy],[ux+Math.cos(a)*(open?26:16),uy+Math.sin(a)*(open?34:29)]],3,.6)}
      g.fillStyle=C(3);g.beginPath();g.arc(ux,uy,2.1,0,TAU);g.fill()
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
        line([[hx-5,hy-6],[hx-13,hy+8],[hx-15+Math.sin(time*3)*2,hy+20]],0,3)
        fill([[sx-8,sy],[sx-3,sy],[bx+6,by+2],[bx+3,by+16],[bx,by+4]],3)
      }
      if(kind==='yanhui'){
        const a=p.of-1.6,h=s.offHand;fill([[h[0],h[1]],[h[0]+Math.cos(a)*28-Math.sin(a)*2,h[1]+Math.sin(a)*28+Math.cos(a)*2],[h[0]+Math.cos(a)*33,h[1]+Math.sin(a)*33],[h[0]+Math.sin(a)*2,h[1]-Math.cos(a)*2]],3)
      }
      arm(s.shoulder,s.elbow,s.hand,kind==='yue'?4.5:3.2);blade(kind==='yue'?58:kind==='yanhui'?43:31,kind==='yue')
    }
    g.restore();const map=v=>[x+v[0]*facing,y+v[1]]
    return {head:map(s.head),tip:map(tip),hand:map(s.hand),neck:map(s.neck),hip:map(s.hip)}
  }
  globalThis.XiabanInkMountainFigures={pose,paint}
})()
