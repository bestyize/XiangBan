/* 灯市角色 · articulated monochrome costume paintings; drawn by the gameplay renderer. */
(() => {
  'use strict'
  const F=globalThis.XiabanInkFigures,TAU=Math.PI*2
  const palette=['#222824','#3c423a','#5c6256','#898b78','#bab8a0','#e4ddc7','#f1e9d5']
  const hash=(a,b)=>{const v=Math.sin(a*127.1+b*311.7)*43758.5453;return v-Math.floor(v)}
  function paint(g,kind,pose,x,y,facing,look={},time=0,seed=0){
    const spec=F.KINDS[kind],s=F.skeleton(spec,pose),entity=look.entity||{}
    const flat=look.color||null,C=i=>flat||palette[i],eye=look.mirror?C(5):'#b73c31'
    const [hx,hy]=s.head,[sx,sy]=s.shoulder,[bx,by]=s.hip
    let tip=s.hand
    g.save();g.translate(x,y);g.scale(facing,1)
    if(look.alpha!==undefined)g.globalAlpha*=look.alpha
    g.lineJoin='round';g.lineCap='round'
    const path=pts=>{
      g.beginPath()
      if(pts.length<6){pts.forEach((p,i)=>i?g.lineTo(...p):g.moveTo(...p))}
      else {
        const soft=.18
        for(let i=0;i<pts.length;i++){
          const p=pts[i],prev=pts[(i+pts.length-1)%pts.length],next=pts[(i+1)%pts.length]
          const a=[p[0]+(prev[0]-p[0])*soft,p[1]+(prev[1]-p[1])*soft],b=[p[0]+(next[0]-p[0])*soft,p[1]+(next[1]-p[1])*soft]
          i?g.lineTo(...a):g.moveTo(...a);g.quadraticCurveTo(p[0],p[1],b[0],b[1])
        }
      }
      g.closePath()
    }
    const stroke=(pts,col=C(1),w=.7)=>{g.strokeStyle=col;g.lineWidth=w;g.beginPath();pts.forEach((p,i)=>i?g.lineTo(...p):g.moveTo(...p));g.stroke()}
    function fabric(pts,shade=2){
      // A few flat ink tones carry the garment; no per-panel gradient, clipping or grain.
      path(pts);g.fillStyle=C(shade);g.fill();g.strokeStyle=C(1);g.lineWidth=.3;g.stroke()
    }
    function seam(pts,shade=4,w=.6){stroke(pts,C(shade),w)}
    function limb(a,b,c,width=4,shade=2){
      const dx=c[0]-a[0],dy=c[1]-a[1],len=Math.hypot(dx,dy)||1,nx=-dy/len,ny=dx/len
      fabric([[a[0]+nx*width,a[1]+ny*width],[b[0]+nx*width*.8,b[1]+ny*width*.8],[c[0]+nx*width*.5,c[1]+ny*width*.5],[c[0]-nx*width*.5,c[1]-ny*width*.5],[b[0]-nx*width*.8,b[1]-ny*width*.8],[a[0]-nx*width,a[1]-ny*width]],shade,6,7)
      seam([[b[0]-nx*2,b[1]-ny*2],[c[0]-nx,c[1]-ny]],shade+1,.5)
    }
    function hand(p){
      fabric([[p[0]-2,p[1]-2],[p[0]+2,p[1]-1],[p[0]+3,p[1]+2],[p[0]+1,p[1]+3],[p[0]-2,p[1]+1]],4,2,0)
      for(let j=0;j<3;j++)stroke([[p[0]+j-1,p[1]],[p[0]+j,p[1]+2]],C(1),.35)
    }
    function boot(leg,back=false){
      limb(s.hip,leg.knee,leg.foot,kind==='watchman'?4.3:3.1,back?1:2)
      const [px,py]=leg.foot
      fabric([[px-3,py-8],[px+2,py-7],[px+3,py-2],[px+7,py-1],[px+8,py+1],[px-4,py+1]],1,4,3)
      seam([[px-2,py-5],[px+2,py-4],[px-2,py-2]],4,.55)
    }
    function ribbon(ax,ay,len,width,shade=4,n=0){
      const tail=-len*(.55+pose.flutter*.3),wave=Math.sin(time*4+n)*2
      fabric([[ax,ay],[ax+width,ay+1],[ax+tail*.4+width,ay+len*.38],[ax+tail+wave+width*.3,ay+len*.7],[ax+tail-4+wave,ay+len*.72-3],[ax+tail*.4-2,ay+len*.4]],shade,n,12)
      seam([[ax+width*.4,ay+2],[ax+tail*.4,ay+len*.4],[ax+tail+wave,ay+len*.68]],Math.min(6,shade+1),.5)
    }
    function sleeve(a,elbow,h,width,tail=0,shade=2){
      const wave=Math.sin(time*4+seed)*1.5
      fabric([[a[0]-4,a[1]-3],[a[0]+5,a[1]-2],[elbow[0]+width*.55,elbow[1]-4],[h[0]+4,h[1]-3],[h[0]+5,h[1]+width*.55],[h[0]-6-tail*.4+wave,h[1]+width+tail],[elbow[0]-width*.7,elbow[1]+width*.65],[a[0]-8,a[1]+8]],shade,8,16)
      seam([[a[0],a[1]+4],[elbow[0]-3,elbow[1]+7],[h[0]-4,h[1]+width+tail-3]],Math.min(6,shade+2),.7)
      seam([[elbow[0]+2,elbow[1]-1],[h[0]+1,h[1]+4],[h[0]-4,h[1]+width]],0,.8)
      seam([[h[0]+3,h[1]-1],[h[0]+2,h[1]+width*.6]],5,1)
      hand(h)
    }
    function face(mask=true){
      fabric([[hx-5,hy-5],[hx+2,hy-6],[hx+5,hy-2],[hx+6.7,hy+1],[hx+4.5,hy+2],[hx+3,hy+6],[hx-2,hy+6],[hx-5,hy+1]],mask?5:3,3,6)
      if(!flat){stroke([[hx+1,hy-2],[hx+4,hy-1.5]],eye,.9+(look.eyeGlow||0)*.5);stroke([[hx+2.8,hy],[hx+2,hy+2],[hx+4,hy+2.5]],C(2),.4);stroke([[hx+1,hy+4],[hx+3,hy+4]],C(1),.4)}
    }
    function robe(shade=2){
      const width=kind==='judge'?16:kind==='mirror'?13:12,flare=pose.flutter*11,wave=Math.sin(time*3+seed)*1.6
      fabric([[sx-8,sy-3],[sx+8,sy-2],[bx+11,by+2],[bx+width+4,1],[bx+4,0],[bx-2,-3],[bx-width-flare,1+wave],[bx-width-5-flare,-2],[bx-11,by+2]],shade,17,28)
      for(let i=0;i<5;i++){
        const xx=bx-9+i*4,end=bx-width+i*(width*2/4)-flare*(1-i/4)
        g.fillStyle=C(i%2?shade:Math.max(0,shade-1));g.beginPath();g.moveTo(xx,by-3);g.quadraticCurveTo(xx+(end-xx)*.7,by*.45,end-2,-1);g.quadraticCurveTo(end,1,end+3,-1);g.quadraticCurveTo(xx+2,by*.5,xx+3,by-3);g.closePath();g.fill()
        seam([[xx+1,by+2],[xx+(end-xx)*.4,by*.4],[end,-2]],Math.min(5,shade+1),.35)
      }
      fabric([[sx-7,sy],[sx-2,sy-4],[bx+6,by-6],[bx+4,by+2],[sx-6,sy+8]],5,4,7)
      fabric([[sx+3,sy-4],[sx+9,sy],[bx-4,by+4],[bx-10,by+1],[sx+1,sy+5]],1,5,10)
      seam([[sx+2,sy-3],[bx-6,by+1]],5,1)
      fabric([[bx-11,by-2],[bx+11,by-3],[bx+12,by+2],[bx-10,by+4]],1,0,3)
      seam([[bx-10,by],[bx+10,by-1]],4,.8)
    }
    function weapon(brush=false,spear=false){
      const [wx,wy]=s.hand,a=pose.sw+(spear?pose.wrist||0:0),len=brush?(entity.judgeStage===3?48:82):spear?86:kind==='lantern'?38:27
      g.save();g.translate(wx,wy);g.rotate(a)
      if(brush&&entity.judgeStage!==3){
        fabric([[-25,-3],[-26,2],[49,4],[50,-4]],1,9,18)
        seam([[-23,-1],[48,-2]],4,.8)
        for(const xx of [-20,8,42,48]){stroke([[xx,-4],[xx,4]],C(4),1);stroke([[xx+1,-4],[xx+1,4]],C(0),.6)}
        fabric([[48,-5],[59,-9],[68,-6],[78,0],[84,4],[72,4],[79,8],[68,7],[73,11],[59,8],[49,5]],5,7,24)
        for(let j=0;j<16;j++){const yy=-5+j*.65,end=70+hash(j,4)*14;g.strokeStyle=C(j%4===0?1:3);g.lineWidth=.35;g.beginPath();g.moveTo(51,yy*.8);g.bezierCurveTo(61,yy-3,end-6,yy+3,end,yy+5);g.stroke()}
        fabric([[72,2],[84,4],[74,5],[79,8],[68,7]],0,3,2)
      }else if(spear){
        fabric([[-27,-1.5],[-27,1.5],[65,2],[65,-2]],2,3,13);seam([[-23,-.7],[63,-.8]],5,.6)
        fabric([[63,-4],[86,0],[64,5],[69,0]],5,9,4);seam([[64,0],[83,0]],1,.6)
        for(let j=0;j<4;j++)stroke([[57+j*2,-2],[57+j*2,2]],C(5),.7)
        for(let j=0;j<2;j++)ribbon(57,3+j*2,12+j*4,1.5,1,j+2)
      }else{
        fabric([[-7,-2],[-7,2],[0,2],[0,-2]],1,2,3)
        if(kind==='lantern'){fabric([[0,-2],[25,-3],[35,-1],[40,5],[32,4],[29,7],[27,2],[0,2]],4,3,6);seam([[1,-2],[25,-3],[35,-1],[40,5]],6,.8)}
        else{fabric([[0,-1.4],[len-6,-1.6],[len,0],[len-4,2],[0,2]],4,3,5);seam([[1,-1],[len-4,-1]],6,.6)}
        stroke([[0,-4],[0,4]],C(2),1.5)
      }
      g.restore();tip=[wx+Math.cos(a)*len,wy+Math.sin(a)*len]
      hand(s.hand)
    }
    // Limbs and garments share the game's skeleton, so the painted feet remain planted.
    boot(s.back,true)
    if(kind==='lantern') {
      ribbon(sx-6,sy+1,39,10,3,1);ribbon(sx-11,sy+8,30,8,4,3)
      boot(s.front);robe(2)
      for(let j=0;j<5;j++){
        const yy=sy+3+j*7
        fabric([[sx-10-j,yy],[sx+7,yy-2],[bx+10+j%2*3,yy+10],[bx-4,yy+6],[sx-17-j,yy+15]],j%2?3:4,j+17,12)
      }
      sleeve(s.offShoulder,s.offElbow,s.offHand,9,4,3)
      // A bent bamboo hanger, long paper lantern and visible segmented frame.
      const px=s.offHand[0]+2,py=s.offHand[1],lx=px+20,ly=py-53
      stroke([[px,py+12],[px-1,py-61],[px+9,py-74],[px+36,py-77]],C(1),2.8)
      seam([[px-.8,py+10],[px-2,py-60],[px+9,py-72],[px+35,py-75]],4,.65)
      for(let j=0;j<5;j++)stroke([[px-3,py-j*12],[px+1,py-j*12]],C(4),1)
      stroke([[lx,py-75],[lx,ly-20]],C(1),.8)
      g.save();g.translate(lx,ly);g.rotate(Math.sin(time*2+seed)*.035)
      g.fillStyle=flat||'#caa16d'
      g.beginPath();g.moveTo(-8,-22);g.bezierCurveTo(-17,-16,-15,18,-8,24);g.quadraticCurveTo(0,28,9,24);g.bezierCurveTo(16,15,17,-16,8,-22);g.closePath();g.fill();g.strokeStyle=C(1);g.lineWidth=.8;g.stroke()
      for(let j=-2;j<=2;j++){g.strokeStyle=flat||'rgba(83,73,48,.45)';g.lineWidth=.4;g.beginPath();g.moveTo(j*3,-22);g.bezierCurveTo(j*6,-8,j*6,10,j*3,24);g.stroke()}
      for(let j=0;j<9;j++)stroke([[-11,-18+j*5],[12,-18+j*5]],flat||'rgba(112,92,56,.38)',.4)
      stroke([[-7,-23],[8,-23]],C(1),2);stroke([[-7,25],[8,25]],C(1),2)
      // Bamboo motif on the translucent paper, just as on the concept lantern.
      stroke([[-2,14],[0,-12]],C(2),.6)
      for(let j=0;j<4;j++){const yy=7-j*5;fabric([[0,yy],[6+(j%2)*2,yy-5],[3,yy]],2,j,0);fabric([[0,yy+1],[-7,yy-2],[-3,yy+2]],2,j,0)}
      ribbon(0,26,12,2,3,1);g.restore()
      face(false)
      fabric([[hx-16,hy-3],[hx-6,hy-10],[hx+1,hy-13],[hx+17,hy-2],[hx+8,hy],[hx-10,hy+1]],2,2,20)
      for(let j=0;j<7;j++)seam([[hx+1,hy-11],[hx-14+j*5,hy-2]],j%2?4:1,.35)
      ribbon(hx-4,hy+2,22,5,3,5)
      sleeve(s.shoulder,s.elbow,s.hand,8,2,2);weapon()
    } else if(kind==='mirror') {
      // Trailing opera silk is made of broad folded cloth, not a triangular cape.
      const tail=bx-34-pose.flutter*12,wave=Math.sin(time*3)*3
      fabric([[sx-7,sy+1],[sx+2,sy+7],[bx+2,by+4],[bx-10,0],[tail-15,3+wave],[tail+2,-4],[tail-8,-6],[bx-15,by-1]],4,19,35)
      seam([[sx-5,sy+7],[bx-11,by+8],[tail-9,1]],5,.8)
      for(let j=0;j<6;j++){
        const xx=tail+10+j*4,yy=-2-j*4
        stroke([[xx,yy+3],[xx+4,yy-7],[xx+7,yy-11]],C(2),.5)
        fabric([[xx+4,yy-6],[xx+10,yy-8],[xx+7,yy-3]],2,j,0)
        fabric([[xx+4,yy-5],[xx-1,yy-10],[xx,yy-4]],3,j,0)
      }
      boot(s.front);robe(2)
      ribbon(bx+4,by,32,4,5,1);ribbon(bx-5,by+2,30,4,3,4)
      sleeve(s.offShoulder,s.offElbow,s.offHand,17,12,4)
      // Large oval bronze hand mirror, with filigree rim and an indistinct ink reflection.
      const mx=s.offHand[0]+4,my=s.offHand[1]-25
      stroke([[mx,my+23],[mx-1,my+33]],C(2),2.6)
      g.save();g.translate(mx,my);g.rotate(-.16)
      g.fillStyle=flat||'#a9b3a2'
      g.beginPath();g.ellipse(0,0,15,24,0,0,TAU);g.fill();g.strokeStyle=C(2);g.lineWidth=2.5;g.stroke()
      g.strokeStyle=C(5);g.lineWidth=.65;g.beginPath();g.ellipse(0,0,12.7,21.5,0,0,TAU);g.stroke()
      for(let j=0;j<18;j++){const a=j/18*TAU;g.fillStyle=C(j%2?3:5);g.beginPath();g.arc(Math.cos(a)*15,Math.sin(a)*24,1,0,TAU);g.fill()}
      if(!flat){fabric([[-6,16],[-3,1],[-6,-4],[0,-8],[4,-3],[2,1],[7,16]],2,4,8);seam([[3,-18],[-2,-3],[3,1],[-4,18]],6,.7)}
      fabric([[-5,-25],[0,-31],[6,-25],[0,-23]],3,5,3);g.restore()
      face();fabric([[hx-6,hy-5],[hx-4,hy-9],[hx+5,hy-8],[hx+7,hy-4],[hx+2,hy-5]],0,1,0)
      // Tangled opera crown, tiny rosettes and long feather curves.
      for(let j=0;j<7;j++){const xx=hx-9+j*2.6,yy=hy-9-Math.sin(j/6*Math.PI)*5;g.strokeStyle=C(3);g.lineWidth=.65;g.beginPath();g.arc(xx,yy,2.2,0,TAU);g.stroke();for(let k=0;k<3;k++){g.fillStyle=C(k%2?3:5);g.beginPath();g.arc(xx+Math.cos(k*2.1)*1.3,yy+Math.sin(k*2.1)*1.3,.8,0,TAU);g.fill()}}
      for(const side of [-1,1]){g.strokeStyle=C(1);g.lineWidth=.7;g.beginPath();g.moveTo(hx+side*5,hy-11);g.bezierCurveTo(hx+side*22,hy-29,hx+side*36,hy-11,hx+side*29+Math.sin(time*2)*2,hy-20);g.stroke();ribbon(hx+side*6,hy-5,15,1.3,4,side+3)}
      sleeve(s.shoulder,s.elbow,s.hand,12,3,3);weapon()
    } else if(kind==='watchman') {
      // A human city watchman: round iron helmet, compact lamellar armor and grounded boots.
      ribbon(sx-8,sy+2,38,11,1,3)
      boot(s.front)
      fabric([[sx-12,sy-3],[sx+11,sy-3],[bx+14,by+3],[bx+17,by+20],[bx+6,by+23],[bx,by+17],[bx-8,by+24],[bx-17,by+20],[bx-13,by+1]],1,7,25)
      // The armor has short, overlapping rectangular lamellae, bound inside a cloth edge.
      for(let row=0;row<6;row++)for(let col=0;col<5;col++) {
        const yy=sy+1+row*5.3,xx=sx-12+col*5+(row%2)*.6
        fabric([[xx,yy],[xx+4.4,yy-.3],[xx+4.6,yy+5.5],[xx,yy+5.8]],row%2?2:1,col+row*5,2)
        seam([[xx+.4,yy+4.7],[xx+4,yy+4.5]],3,.45)
        if(!flat&&row%2===0){g.fillStyle=C(4);for(const off of [2.2]){g.beginPath();g.arc(xx+off,yy+1.2,.34,0,TAU);g.fill()}}
      }
      fabric([[sx-13,sy-1],[sx-9,sy-2],[bx+8,by+5],[bx+4,by+6]],2,4,6)
      seam([[sx-11,sy],[bx+6,by+5]],4,.6)
      fabric([[bx-14,by+3],[bx+14,by+2],[bx+14,by+7],[bx-14,by+8]],0,2,5)
      fabric([[bx-2,by+3],[bx+3,by+3],[bx+3,by+7],[bx-2,by+7]],3,1,0)
      // A small patrol tally identifies the guard without an abstract mask or paper crown.
      stroke([[bx+8,by+5],[bx+9,by+12]],C(2),.7)
      fabric([[bx+6,by+11],[bx+12,by+11],[bx+12,by+20],[bx+6,by+21]],4,3,4)
      if(!flat){g.save();g.translate(bx+9,by+17);g.scale(facing,1);g.fillStyle=C(1);g.font='5px KaiTi,serif';g.textAlign='center';g.fillText('巡',0,0);g.restore()}
      // Rounded shoulder guards follow the arm sockets instead of spreading into triangular wings.
      for(const [ax,ay] of [s.offShoulder,s.shoulder]) {
        g.fillStyle=C(1);g.strokeStyle=C(3);g.lineWidth=.65
        g.beginPath();g.ellipse(ax,ay+3,10,6.5,-.15,Math.PI,TAU);g.lineTo(ax+10,ay+6);g.quadraticCurveTo(ax,ay+11,ax-10,ay+6);g.closePath();g.fill();g.stroke()
        for(let j=0;j<3;j++){g.strokeStyle=C(3);g.lineWidth=.5;g.beginPath();g.moveTo(ax-8,ay+3+j*2);g.quadraticCurveTo(ax,ay+7+j*2,ax+8,ay+3+j*2);g.stroke()}
      }
      sleeve(s.offShoulder,s.offElbow,s.offHand,7,0,2)
      face(false)
      // Dome, brow band and cheek/neck protection keep a recognisable human profile.
      g.fillStyle=C(1);g.strokeStyle=C(3);g.lineWidth=.6
      g.beginPath();g.moveTo(hx-8,hy-2);g.bezierCurveTo(hx-10,hy-14,hx+7,hy-17,hx+8,hy-3);g.quadraticCurveTo(hx,hy-5,hx-8,hy-2);g.closePath();g.fill();g.stroke()
      g.strokeStyle=C(3);g.lineWidth=.9;g.beginPath();g.moveTo(hx-1,hy-13);g.quadraticCurveTo(hx+2,hy-8,hx+2,hy-4);g.stroke()
      fabric([[hx-9,hy-3],[hx+9,hy-4],[hx+10,hy-1],[hx-9,hy]],2,3,6)
      fabric([[hx-8,hy-1],[hx-4,hy],[hx-4,hy+8],[hx-9,hy+10],[hx-11,hy+3]],1,4,7)
      for(let j=0;j<3;j++)seam([[hx-9,hy+2+j*2],[hx-5,hy+1+j*2]],3,.5)
      seam([[hx+4,hy+4],[hx+1,hy+7],[hx-4,hy+6]],1,.6)
      sleeve(s.shoulder,s.elbow,s.hand,7,0,2)
      for(const [a,h] of [[s.elbow,s.hand],[s.offElbow,s.offHand]]) {
        const dx=h[0]-a[0],dy=h[1]-a[1]
        for(let j=0;j<3;j++){const q=.3+j*.2;stroke([[a[0]+dx*q-2,a[1]+dy*q],[a[0]+dx*q+2,a[1]+dy*q+1]],C(3),.7)}
      }
      weapon(false,true)
    } else if(kind==='judge') {
      ribbon(sx-9,sy-1,57,15,1,1);ribbon(sx-5,sy+1,53,9,3,4)
      boot(s.front);robe(1)
      // Long pale stole, angular lapels and separate hanging paper slips.
      fabric([[sx-1,sy+2],[sx+4,sy+2],[bx+9,by-3],[bx+7,-1],[bx+2,-3],[bx+3,by],[sx-2,sy+7]],5,4,22)
      fabric([[sx-7,sy+3],[sx-2,sy+6],[bx-3,by],[bx-6,-2],[bx-11,1],[bx-8,by-1]],3,6,15)
      for(let j=0;j<3;j++){const xx=bx-4+j*4;fabric([[xx,by],[xx+2.4,by],[xx+1+Math.sin(time*3+j),by+19+j*3],[xx-1,by+17+j*3]],5,j,6);for(let n=0;n<3;n++)seam([[xx,by+3+n*4],[xx+1,by+4+n*4]],2,.4)}
      sleeve(s.offShoulder,s.offElbow,s.offHand,19,6,2)
      face();fabric([[hx-7,hy-5],[hx-8,hy-24],[hx-3,hy-27],[hx+7,hy-25],[hx+8,hy-5]],1,8,20)
      fabric([[hx-7,hy-23],[hx-2,hy-25],[hx-1,hy-7],[hx-6,hy-5]],3,7,9)
      seam([[hx+1,hy-24],[hx+4,hy-7]],4,.65)
      for(const side of [-1,1])fabric([[hx+side*6,hy-6],[hx+side*25,hy-9],[hx+side*26,hy-6],[hx+side*7,hy-3]],1,side+4,5)
      fabric([[hx-2,hy-5],[hx+4,hy-5],[hx+3,hy-2],[hx,hy-1]],4,1,0)
      sleeve(s.shoulder,s.elbow,s.hand,17,6,1);weapon(true)
    }
    g.restore()
    return {head:[x+hx*facing,y+hy],tip:[x+tip[0]*facing,y+tip[1]],hand:[x+s.hand[0]*facing,y+s.hand[1]],neck:[x+s.neck[0]*facing,y+s.neck[1]],hip:[x+bx*facing,y+by]}
  }
  // Rasterize fine costume paths inside a small surface. Position remains full-frame-rate;
  // cloth poses update at 30 Hz and immediately on attack, stagger, flash or phase changes.
  const frames=new WeakMap(),scale=2,ox=150,oy=180,width=320,height=220
  const stats={paints:0,blits:0}
  function draw(g,kind,pose,x,y,facing,look={},time=0,seed=0){
    const key=look.entity||pose
    let frame=frames.get(key)
    if(!frame){const node=document.createElement('canvas');node.width=width*scale;node.height=height*scale;frame={node,ctx:node.getContext('2d'),next:0,out:null,signature:''};frames.set(key,frame)}
    const signature=kind+':'+(look.entity?.state||'')+':'+(look.entity?.judgeStage||0)+':'+(look.color||'')+':'+Boolean(look.mirror)
    const now=performance.now()
    if(!frame.out||now>=frame.next||signature!==frame.signature){
      const c=frame.ctx;c.setTransform(1,0,0,1,0,0);c.clearRect(0,0,frame.node.width,frame.node.height);c.setTransform(scale,0,0,scale,ox*scale,oy*scale)
      frame.out=paint(c,kind,pose,0,0,1,{...look,alpha:1},time,seed)
      frame.next=now+1000/30;frame.signature=signature;stats.paints++
    }
    g.save();g.translate(x,y);g.scale(facing,1);if(look.alpha!==undefined)g.globalAlpha*=look.alpha
    g.drawImage(frame.node,-ox,-oy,width,height);g.restore();stats.blits++
    const map=p=>p&&[x+p[0]*facing,y+p[1]]
    return {head:map(frame.out.head),tip:map(frame.out.tip),hand:map(frame.out.hand),neck:map(frame.out.neck),hip:map(frame.out.hip)}
  }
  globalThis.XiabanInkLanternFigures={draw,stats}
})()
