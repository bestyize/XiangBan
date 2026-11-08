/* 灯市卷 · cached hand-painted architecture, paper costumes and readable mirror/brush effects. */
(() => {
  'use strict'
  const A=globalThis.XiabanInkArt,F=globalThis.XiabanInkFigures,G=globalThis.XiabanInkBlade.GROUND
  const TAU=Math.PI*2,ink='#262725',paper='#ece6d6',red='#b8322a'
  function line(g,pts,color=ink,w=1){g.strokeStyle=color;g.lineWidth=w;g.beginPath();pts.forEach((p,i)=>i?g.lineTo(...p):g.moveTo(...p));g.stroke()}
  function roof(g,x,y,w,h=26){
    const wash=g.createLinearGradient(0,y-h,0,y+18);wash.addColorStop(0,'#30322f');wash.addColorStop(1,'#686b63');g.fillStyle=wash;g.beginPath();g.moveTo(x-w/2-14,y+9);g.quadraticCurveTo(x-w/2+18,y+5,x-w*0.28,y-h);g.lineTo(x+w*0.28,y-h);g.quadraticCurveTo(x+w/2-15,y+4,x+w/2+14,y+7);g.lineTo(x+w/2-2,y+18);g.quadraticCurveTo(x,y+7,x-w/2-12,y+19);g.closePath();g.fill()
    for(let i=0;i<Math.floor(w/9);i++){const xx=x-w*.43+i*9;line(g,[[xx*.95+x*.05,y-h+5],[xx,y+9]],'rgba(191,207,190,0.32)',.8)}
    line(g,[[x-w*.28,y-h],[x+w*.28,y-h]],'rgba(219,225,204,0.6)',2)
    for(let i=0;i<Math.floor(w/7);i++){const xx=x-w*.48+i*7,yy=y+11+Math.pow((xx-x)/(w*.5),2)*4;g.strokeStyle='rgba(23,41,33,0.65)';g.lineWidth=1;g.beginPath();g.arc(xx,yy,2.3,0,Math.PI);g.stroke()}
    line(g,[[x-w*.49,y+17],[x,y+10],[x+w*.49,y+16]],'rgba(212,221,193,0.5)',1)
    for(const side of [-1,1]){g.strokeStyle=ink;g.lineWidth=2;g.beginPath();g.moveTo(x+side*(w/2+10),y+8);g.quadraticCurveTo(x+side*(w/2+23),y-3,x+side*(w/2+15),y-8);g.stroke()}
  }
  function lamp(g,x,y,w=18,h=31,t=0,warm=false){
    g.save();g.translate(x,y);g.rotate(Math.sin(t)*.035)
    line(g,[[0,-16],[0,-h*.5]],'rgba(31,47,45,0.7)',1)
    const wash=g.createLinearGradient(-w/2,0,w/2,0);wash.addColorStop(0,warm?'#915343':'#c2c0ac');wash.addColorStop(.45,warm?'#e4b575':'#f5efde');wash.addColorStop(1,warm?'#b97b50':'#ded8c3')
    g.fillStyle=wash;g.strokeStyle='rgba(40,59,54,0.75)';g.lineWidth=.8;g.beginPath();g.ellipse(0,0,w/2,h/2,0,0,TAU);g.fill();g.stroke()
    for(let i=-2;i<=2;i++){g.strokeStyle='rgba(79,98,77,0.3)';g.beginPath();g.ellipse(i*w*.05,0,w*(.15+Math.abs(i)*.12),h/2-.5,0,-Math.PI/2,Math.PI/2);g.stroke()}
    for(let i=-2;i<=2;i++)line(g,[[-w*.43,i*h*.14],[w*.43,i*h*.14]],'rgba(94,112,87,0.28)',.5)
    g.fillStyle=warm?'#765044':'#666457';g.fillRect(-w*.25,-h/2-1,w*.5,3);g.fillRect(-w*.25,h/2-2,w*.5,3)
    line(g,[[0,h/2],[Math.sin(t*2)*2,h/2+9]],'rgba(61,80,67,0.8)',1.3);g.restore()
  }
  function building(g,x,base,w,h,seed=0){
    const floors=h>140?2:1
    g.fillStyle='rgba(131,131,116,0.24)';g.fillRect(x-w/2,base-h,w,h)
    for(let side of [-1,1]){g.fillStyle='rgba(52,53,47,0.8)';g.fillRect(x+side*(w/2-5)-3,base-h,6,h)}
    for(let floor=0;floor<floors;floor++){
      const yy=base-floor*h/floors,fh=h/floors
      line(g,[[x-w/2,yy-5],[x+w/2,yy-5]],ink,4)
      for(let j=0;j<4;j++){
        const xx=x-w*.4+j*w*.22,ww=w*.16
        g.fillStyle=j===1&&floor===0?'rgba(27,43,43,0.75)':'rgba(231,225,207,0.66)';g.fillRect(xx,yy-fh+19,ww,fh-27)
        for(let k=1;k<4;k++)line(g,[[xx+k*ww/4,yy-fh+20],[xx+k*ww/4,yy-9]],'rgba(37,56,49,0.65)',1)
        for(let k=0;k<4;k++)line(g,[[xx,yy-fh+27+k*(fh-35)/4],[xx+ww,yy-fh+27+k*(fh-35)/4]],'rgba(37,56,49,0.6)',1)
      }
      if(floor>0)roof(g,x,yy+6,w+12,14)
    }
    roof(g,x,base-h,w+18,25)
    // Split timber rails, tenon ends and battered paper placards.
    for(const side of [-1,1]) {
      const xx=x+side*(w/2-5)
      for(let j=0;j<8;j++){const yy=base-h+20+j*h/9;line(g,[[xx-1,yy],[xx-2,yy+8],[xx,yy+14]],'rgba(211,221,190,0.23)',.6)}
      g.fillStyle='rgba(39,60,44,0.65)';g.fillRect(xx-8,base-h+12,16,4)
    }
    if(w>140){const xx=x-w*.33;g.fillStyle='rgba(225,225,193,0.7)';g.beginPath();g.moveTo(xx-7,base-h+24);g.lineTo(xx+8,base-h+23);g.lineTo(xx+7,base-h+65);g.lineTo(xx,base-h+62);g.lineTo(xx-7,base-h+66);g.closePath();g.fill();g.save();g.fillStyle='rgba(35,58,42,0.7)';g.textAlign='center';g.font='11px KaiTi,serif';g.fillText(seed%2?'布':'茶',xx,base-h+39);g.fillText(seed%2?'庄':'肆',xx,base-h+54);g.restore()}
    line(g,[[x-w*.5,base],[x+w*.5,base]],'#444742',5)
    for(let j=0;j<5;j++){const xx=x-w*.45+j*w*.2;line(g,[[xx,base-5],[xx-3,base+8]],'rgba(36,53,46,0.45)',.8)}
    if(seed%2===0)lamp(g,x+w*.38,base-h+40,15,25,0,seed%3===0)
  }
  function landmark(kind,res,seed){
    const sizes={market:[650,290],paper:[490,275],mirror:[610,290],theatre:[630,295],court:[690,310]}
    const [w,h]=sizes[kind]
    return A.sprite(w,h,res,g=>{
      g.lineCap='round';g.lineJoin='round'
      if(kind==='market'||kind==='paper') {
        building(g,-w*.29,-5,160,173,seed);building(g,-w*.02,-5,180,215,seed+1);building(g,w*.29,-5,175,153,seed+2)
        for(const x of [-w*.31,w*.2]){
          g.fillStyle='rgba(234,227,207,0.92)';g.beginPath();g.moveTo(x-70,-112);g.lineTo(x+63,-116);g.lineTo(x+80,-77);g.quadraticCurveTo(x+20,-63,x-80,-76);g.closePath();g.fill()
          for(let i=0;i<11;i++)line(g,[[x-68+i*13,-110],[x-79+i*15,-76]],'rgba(59,81,66,0.3)',.7)
          line(g,[[x-77,-76],[x-77,0]],ink,3);line(g,[[x+77,-76],[x+77,0]],ink,3)
          g.fillStyle='#78796d';g.fillRect(x-61,-35,120,30)
          for(let i=0;i<7;i++){g.fillStyle='rgba(222,225,199,0.6)';g.beginPath();g.ellipse(x-45+i*15,-39,5,7,0,0,TAU);g.fill()}
        }
        if(kind==='paper') {
          g.save();g.translate(108,-8);g.fillStyle='#e5dfcc';g.strokeStyle='#616155';g.lineWidth=2
          g.beginPath();g.moveTo(-54,-51);g.lineTo(18,-59);g.lineTo(29,-108);g.lineTo(51,-121);g.lineTo(63,-105);g.lineTo(49,-87);g.lineTo(40,-48);g.lineTo(49,-2);g.lineTo(34,0);g.lineTo(17,-38);g.lineTo(-23,-37);g.lineTo(-36,0);g.lineTo(-49,-1);g.lineTo(-45,-42);g.closePath();g.fill();g.stroke()
          for(let i=0;i<7;i++)line(g,[[-42+i*10,-51],[-38+i*10,-36]],'rgba(57,81,64,0.6)',1)
          line(g,[[29,-105],[42,-103],[40,-58],[24,-40]],'rgba(57,81,64,0.6)',1)
          g.beginPath();g.arc(47,-105,2,0,TAU);g.fillStyle=ink;g.fill();g.restore()
        }
      } else if(kind==='mirror') {
        building(g,-220,-15,110,123,2);building(g,225,-12,120,151,3)
        for(let side of [-1,1])line(g,[[side*118,0],[side*122,-240]],'#50524b',8)
        line(g,[[-151,-236],[147,-236]],ink,7)
        for(const [x,y,ww,hh] of [[0,-137,123,157],[-149,-114,38,55],[153,-105,43,62]]) {
          line(g,[[x,-234],[x,y-hh/2]],'rgba(37,53,48,0.7)',2)
          const wash=g.createLinearGradient(x-ww/2,y-hh/2,x+ww/2,y+hh/2);wash.addColorStop(0,'#8caaa0');wash.addColorStop(.5,'#d5dfca');wash.addColorStop(1,'#6e938a')
          g.fillStyle=wash;g.strokeStyle='#526960';g.lineWidth=5;g.beginPath();g.ellipse(x,y,ww/2,hh/2,0,0,TAU);g.fill();g.stroke()
          g.strokeStyle='rgba(219,226,201,0.85)';g.lineWidth=1;g.beginPath();g.ellipse(x,y,ww/2-4,hh/2-4,0,0,TAU);g.stroke()
          g.save();g.beginPath();g.ellipse(x,y,ww/2-6,hh/2-6,0,0,TAU);g.clip();g.translate(x,y+hh*.3);g.scale(.26,-.26);building(g,0,0,170,200,2);g.restore()
          line(g,[[x+10,y-hh*.3],[x-4,y-5],[x+8,y+5],[x-8,y+hh*.35]],'rgba(242,243,223,0.85)',1.5)
          for(let j=0;j<12;j++){const a=j/12*TAU;g.fillStyle='#708275';g.beginPath();g.arc(x+Math.cos(a)*(ww/2+3),y+Math.sin(a)*(hh/2+3),2,0,TAU);g.fill()}
        }
      } else {
        const court=kind==='court'
        g.fillStyle='rgba(117,121,110,0.25)';g.fillRect(-255,-228,510,228)
        for(let side of [-1,1]){g.fillStyle='#484d46';g.fillRect(side*235-9,-230,18,230);building(g,side*285,0,95,169,5)}
        roof(g,0,-226,551,40);roof(g,0,-255,320,27)
        for(let x=-190;x<=190;x+=76){line(g,[[x,-203],[x,-5]],'rgba(34,55,46,0.7)',5);for(let j=0;j<7;j++)line(g,[[x-28,-184+j*18],[x+28,-184+j*18]],'rgba(52,75,59,0.45)',1)}
        if(court) {
          for(let i=0;i<5;i++){
            const x=-180+i*90, hh=i%2?140:190
            g.fillStyle='rgba(238,232,215,0.86)';g.beginPath();g.moveTo(x-24,-213);g.lineTo(x+24,-213);g.lineTo(x+22,-213+hh);g.lineTo(x+8,-218+hh);g.lineTo(x-4,-209+hh);g.lineTo(x-24,-214+hh);g.closePath();g.fill()
            for(let j=0;j<7;j++){const yy=-192+j*17;if(yy>-220+hh)break;line(g,[[x-9,yy],[x+7,yy+3],[x-1,yy+9]],'rgba(56,78,65,0.28)',1.3)}
          }
        } else {
          for(let side of [-1,1]){g.fillStyle='#77786b';g.beginPath();g.moveTo(side*235,-205);g.lineTo(side*95,-205);g.quadraticCurveTo(side*115,-100,side*218,-40);g.lineTo(side*235,-40);g.closePath();g.fill();for(let j=0;j<5;j++)line(g,[[side*(100+j*24),-201],[side*(204+j*4),-48]],'rgba(199,214,185,0.24)',1)}
          g.fillStyle='rgba(232,229,206,0.65)';g.fillRect(-97,-185,194,140)
          for(let i=0;i<3;i++){const x=-50+i*50;g.fillStyle='rgba(32,57,48,0.32)';g.beginPath();g.arc(x,-111-i%2*14,6,0,TAU);g.fill();line(g,[[x,-105-i%2*14],[x+4,-71],[x-11,-51]],'rgba(32,57,48,0.32)',5)}
        }
        for(let i=0;i<4;i++){g.fillStyle=i%2?'#96988a':'#797d71';g.fillRect(-265-i*8,-4+i*6,530+i*16,6);line(g,[[-265-i*8,-4+i*6],[265+i*8,-4+i*6]],'rgba(218,227,201,0.4)',1)}
      }
    },w/2,h-20)
  }
  function build(r,span){
    const out={}
    for(const [k,f,base,alpha] of [['far',.06,248,.18],['mid',.16,270,.25],['near',.32,288,.3]])out[k]=A.mountains({width:span(f),top:70,height:220,base,res:r(.35),seed:501+base,peakW:[70,180],peakH:[25,100],gap:[.6,1.2],round:2,detail:14,detailAmp:6,color:[86,91,86],alpha,depth:40,strokeCell:4,strokeDensity:.45,strokeLen:8,outline:.5,trees:.15,fadeH:70})
    out.city={};for(const [i,k] of ['market','paper','mirror','theatre','court'].entries())out.city[k]=landmark(k,r(.85),i+31)
    return out
  }
  const props=[['market',830],['paper',1770],['mirror',2750],['theatre',3740],['court',4800],['court',5780]]
  function platform(p,res) {
    const width=p.w,height=G-p.y+25
    return A.sprite(width+28,height+25,res,g=>{
      g.lineJoin='round'
      g.fillStyle='#9c9e90';g.strokeStyle='#5b6056';g.lineWidth=1
      g.beginPath();g.moveTo(0,0);g.lineTo(width,0);g.lineTo(width,G-p.y+2);g.lineTo(width-18,G-p.y+2);g.bezierCurveTo(width-28,12,28,12,18,G-p.y+2);g.lineTo(0,G-p.y+2);g.closePath();g.fill();g.stroke()
      for(let j=0;j<7;j++){const q=j/6,x=width*.1+q*width*.8;line(g,[[x,3],[x+(q-.5)*12,18+Math.pow(Math.abs(q-.5)*2,2)*(G-p.y-18)]],'rgba(36,58,44,0.42)',.8)}
      for(let i=0;i<5;i++){const x=i*width/4;g.fillStyle='#73776a';g.fillRect(x-2,-22,4,24);g.beginPath();g.arc(x,-22,3,0,TAU);g.fill()}
      line(g,[[0,-13],[width,-13]],'#848a7a',2);line(g,[[0,0],[width,0]],'rgba(225,232,204,0.8)',2)
    },14,25)
  }
  function background(g,a,cam,clock){
    const scene=globalThis.XiabanInkScene;scene.layer(g,cam,.38)
    for(let i=0;i<7;i++){g.save();g.translate(i*310+80,280);g.scale(.56,.56);A.drawSprite(g,a.city[i%2?'paper':'market'],0,0,.2);g.restore()}
  }
  function ground(g,a,cam,clock){
    const left=cam.left-60,right=cam.left+cam.viewW+60
    // Faint inverted architecture is contained below the walking line.
    for(const [k,x]of props){if(x+400<left||x-400>right)continue
      g.save();g.translate(0,G*2+12);g.scale(1,-.28);A.drawSprite(g,a.city[k],x,G,.22);g.restore()
      A.drawSprite(g,a.city[k],x,G+4,k==='mirror'?.66:k==='court'?.57:.68)
    }
    const wash=g.createLinearGradient(0,G,0,G+75);wash.addColorStop(0,'rgba(115,116,105,0.22)');wash.addColorStop(1,'rgba(160,165,150,0.04)');g.fillStyle=wash;g.fillRect(left,G,right-left,95)
    for(let j=0;j<5;j++){const yy=G+j*9;line(g,[[left,yy],[right,yy]],'rgba(229,233,209,0.37)',.8);for(let x=Math.floor(left/82)*82+(j%2)*39;x<right;x+=82)line(g,[[x,yy],[x+3,yy+9]],'rgba(34,59,46,0.33)',.7)}
    line(g,[[left,G],[right,G]],'#4b4d44',2)
    for(let x=Math.floor(left/240)*240;x<right+50;x+=240){if(x>5200)continue;line(g,[[x,G],[x,G-140]],'rgba(33,54,44,0.55)',3);line(g,[[x,G-139],[x+35,G-144]],'rgba(33,54,44,0.6)',2);lamp(g,x+32,G-112,17,29,clock+x,Math.floor(x/240)%3===0)}
  }
  function foreground(g,a,cam,clock){
    if(cam.left>5100)return
    const y=cam.top+18
    g.save();line(g,[[cam.left-30,y],[cam.left+cam.viewW*.5,y+10],[cam.left+cam.viewW+30,y-3]],'rgba(31,53,41,0.22)',1)
    for(const x of [cam.left+14,cam.left+cam.viewW-18]){g.globalAlpha=.3;lamp(g,x,y+36,26,42,clock*.8+x)}g.restore()
  }
  function costume(g,kind,s,pose,look,time,entity){
    const mark=look.mirror?'#dbe6cf':red, color=look.flash?look.color:ink,[hx,hy]=s.head,[sx,sy]=s.shoulder
    g.save();g.fillStyle=color;g.strokeStyle=color;g.lineJoin='round'
    if(kind==='mirror') {
      // A high opera crown and one enormous sleeve are asymmetric even at thumbnail scale.
      for(let i=0;i<5;i++){const x=hx-8+i*4;line(g,[[x,hy-4],[x-2,hy-13-i%2*4]],color,1);g.beginPath();g.arc(x-2,hy-13-i%2*4,2,0,TAU);g.fill()}
      g.fillStyle=look.flash?color:paper;g.beginPath();g.ellipse(hx+1,hy+1,4.6,6,0,0,TAU);g.fill()
      line(g,[[hx+1,hy-1],[hx+5,hy]],mark,1.2)
      g.fillStyle=color;g.beginPath();g.moveTo(s.offShoulder[0],s.offShoulder[1]);g.lineTo(s.offHand[0]+4,s.offHand[1]);g.quadraticCurveTo(s.offHand[0]-3,s.offHand[1]+33,s.hip[0]-34+Math.sin(time*4)*3,s.hip[1]+17);g.quadraticCurveTo(s.hip[0]-18,sy+13,s.offShoulder[0],s.offShoulder[1]);g.fill()
      const x=s.offHand[0]-5,y=s.offHand[1]-16
      g.fillStyle=look.flash?color:'#939e92';g.strokeStyle=color;g.lineWidth=2.5;g.beginPath();g.ellipse(x,y,12,18,-.25,0,TAU);g.fill();g.stroke();line(g,[[x,y+17],[x+2,y+25]],color,2)
      line(g,[[x+1,y-13],[x-4,y],[x+3,y+5],[x-2,y+12]],'rgba(222,230,205,0.8)',.8)
    } else if(kind==='paper') {
      const open=entity?.state==='stagger'
      if(open)for(let i=0;i<6;i++)line(g,[[sx-10,sy+i*5],[sx+12,sy+i*5+2]],'#d1c9ad',1.4)
      for(let row=0;row<4;row++)for(let col=0;col<3;col++){
        if(open&&col===1)continue
        const x=sx-19+col*12-row*.8+(open?(col-1)*11:0),y=sy+row*9
        g.fillStyle=look.flash?color:row%2?'#bab9a6':'#e2dcc8';g.strokeStyle='#5a6051';g.lineWidth=.8;g.beginPath();g.moveTo(x,y);g.lineTo(x+12,y-2);g.lineTo(x+16,y+10);g.lineTo(x+5,y+8);g.closePath();g.fill();g.stroke()
        line(g,[[x+3,y+1],[x+8,y+7]],'rgba(46,67,49,0.4)',.7)
      }
      g.fillStyle=look.flash?color:paper;g.beginPath();g.moveTo(hx-8,hy-8);g.lineTo(hx+8,hy-9);g.lineTo(hx+7,hy+8);g.lineTo(hx-4,hy+9);g.closePath();g.fill();line(g,[[hx-2,hy-8],[hx+2,hy+8]],'#727361',1.3)
      line(g,[[hx+3,hy-1],[hx+7,hy-1]],mark,1.5)
      for(const side of [-1,1])line(g,[[sx+side*10,sy-2],[sx+side*28,sy-5],[sx+side*22,sy+9]],color,3)
    } else if(kind==='lantern') {
      const x=s.offHand[0]-7,y=s.offHand[1]-45
      line(g,[[s.offHand[0],s.offHand[1]+15],[x-5,y-30],[x+21,y-33]],color,2.3)
      lamp(g,x+17,y-7,18,34,time*3,true)
      g.fillStyle=color;g.beginPath();g.moveTo(hx-14,hy-2);g.lineTo(hx-2,hy-12);g.lineTo(hx+13,hy-1);g.closePath();g.fill()
    } else if(kind==='judge') {
      const sword=entity?.judgeStage===3
      g.fillStyle=color;g.beginPath();g.moveTo(hx-8,hy-2);g.lineTo(hx-9,hy-24);g.lineTo(hx+7,hy-26);g.lineTo(hx+9,hy-1);g.closePath();g.fill();line(g,[[hx-25,hy-4],[hx+25,hy-7]],color,2)
      g.fillStyle=look.flash?color:paper;g.beginPath();g.ellipse(hx+2,hy+2,4.8,7.2,-.1,0,TAU);g.fill();line(g,[[hx+2,hy],[hx+6,hy-1]],mark,1.3)
      for(const hand of [s.hand,s.offHand]){g.fillStyle=color;g.beginPath();g.moveTo(sx-8,sy);g.lineTo(hand[0]+5,hand[1]-3);g.lineTo(hand[0]+3,hand[1]+26);g.quadraticCurveTo(s.hip[0]-5,s.hip[1]+12,sx-8,sy);g.fill();line(g,[[hand[0],hand[1]+4],[hand[0]-5,hand[1]+26]],'rgba(224,226,202,0.4)',1)}
      g.save();g.translate(...s.hand);g.rotate(pose.sw)
      if(sword){g.fillStyle=paper;g.beginPath();g.moveTo(-6,-2);g.lineTo(40,-1);g.lineTo(49,0);g.lineTo(38,2);g.lineTo(-6,2);g.fill()}
      else {line(g,[[-22,0],[45,0]],color,5);line(g,[[-19,-1],[43,-1]],'rgba(204,218,193,0.65)',1);g.fillStyle=paper;g.beginPath();g.moveTo(43,-6);g.quadraticCurveTo(58,-10,75,3);g.lineTo(62,2);g.lineTo(69,6);g.quadraticCurveTo(48,8,43,5);g.fill();g.fillStyle=color;g.beginPath();g.moveTo(65,0);g.lineTo(79,4);g.lineTo(66,3);g.lineTo(71,7);g.lineTo(60,5);g.fill()}
      g.restore()
    }
    g.restore()
  }
  function effects(g,state,clock){
    const r=state.reflection
    if(r){g.save();g.translate(r.x,r.y);g.strokeStyle=r.state==='tell'?red:paper;g.lineWidth=r.state==='tell'?2:4
      if(r.state==='tell'){const q=1-r.t/r.total;g.beginPath();g.arc(0,0,20,-2.2,-2.2+q*5.2);g.stroke();g.font='bold 15px KaiTi,serif';g.textAlign='center';g.fillStyle=red;g.fillText('返',0,-29)}
      else{g.beginPath();g.moveTo(-r.dir*22,5);g.quadraticCurveTo(-r.dir*8,-10,r.dir*16,0);g.stroke();line(g,[[-r.dir*28,8],[r.dir*8,3]],red,1)}g.restore()
    }
    for(const w of state.writing){g.save();const tell=w.phase==='tell',alpha=w.phase==='fade'?Math.max(0,w.t/.4):1;g.globalAlpha=alpha
      for(const seg of w.segments){const horizontal=seg.x1-seg.x0>seg.y1-seg.y0
        g.fillStyle=tell?'rgba(182,50,42,0.08)':'rgba(23,44,35,0.75)';g.fillRect(seg.x0,seg.y0,seg.x1-seg.x0,seg.y1-seg.y0)
        g.setLineDash(tell?[7,7]:[]);g.strokeStyle=tell?'rgba(184,50,42,0.75)':'#142b23';g.lineWidth=tell?1.7:4;g.strokeRect(seg.x0+1,seg.y0+1,seg.x1-seg.x0-2,seg.y1-seg.y0-2)
        if(!tell)for(let i=0;i<5;i++){const q=i/5;line(g,horizontal?[[seg.x0+5,seg.y0+q*(seg.y1-seg.y0)],[seg.x1-8,seg.y0+q*(seg.y1-seg.y0)+2]]:[[seg.x0+q*(seg.x1-seg.x0),seg.y0+5],[seg.x0+q*(seg.x1-seg.x0)+2,seg.y1-4]],'rgba(211,223,198,0.2)',.7)}
      }
      g.setLineDash([])
      if(tell){g.fillStyle=red;g.font='bold 20px KaiTi,serif';g.textAlign='center';g.fillText('危',w.kind==='horizontal'?w.x:w.x+44,G-65);g.fillStyle=paper;g.font='14px KaiTi,serif';if(w.kind==='box')g.fillText('← 缺口',w.x-110,G-55)}
      g.restore()
    }
  }
  globalThis.XiabanInkLantern={build,platform,background,ground,foreground,costume,effects}
})()
