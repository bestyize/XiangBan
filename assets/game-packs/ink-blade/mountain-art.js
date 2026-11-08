/* 悬山卷 · static ink cliffs and architecture, cinnabar sun/berries/flowers; tiny moving details. */
(() => {
  'use strict'
  const A=globalThis.XiabanInkArt,G=globalThis.XiabanInkBlade.GROUND,TAU=Math.PI*2
  const ink='#292d2b',stone='#96988c',light='#ddd9c9',red='#b8322a'
  const line=(g,pts,c=ink,w=1)=>{g.strokeStyle=c;g.lineWidth=w;g.beginPath();pts.forEach((p,i)=>i?g.lineTo(...p):g.moveTo(...p));g.stroke()}
  function roof(g,x,y,w){
    g.fillStyle='#3f4440';g.beginPath();g.moveTo(x-w/2-12,y+12);g.quadraticCurveTo(x-w*.35,y+4,x-w*.28,y-24);g.lineTo(x+w*.28,y-24);g.quadraticCurveTo(x+w*.36,y+7,x+w/2+12,y+7);g.lineTo(x+w/2,y+19);g.quadraticCurveTo(x,y+8,x-w/2-12,y+22);g.closePath();g.fill()
    for(let i=0;i<w/10;i++){const xx=x-w*.43+i*10;line(g,[[xx*.8+x*.2,y-19],[xx,y+11]],'rgba(221,220,201,.36)',.7)}
    line(g,[[x-w*.28,y-24],[x+w*.28,y-24]],'#c9c7b6',1.5)
  }
  function cliff(g,cx,base,w,h,seed){
    const rand=A.rng(seed),pts=[]
    for(let j=0;j<15;j++){const q=j/14;pts.push([cx-w/2+q*w,base-h*Math.pow(Math.sin(q*Math.PI),.3)*(0.88+rand()*.12)])}
    g.fillStyle='#6b7169';g.beginPath();g.moveTo(cx-w/2,base+20);pts.forEach(p=>g.lineTo(...p));g.lineTo(cx+w/2,base+20);g.closePath();g.fill()
    for(let i=0;i<95;i++){const x=cx-w*.4+rand()*w*.8,hh=h*(.45+rand()*.5),y=base-hh;line(g,[[x,y],[x-7,y+hh*.28],[x+4,y+hh*.55],[x-11,base-3]],i%3?'rgba(216,218,199,.18)':'rgba(34,43,37,.28)',.7+rand()*2)}
    line(g,pts,'rgba(35,42,35,.8)',1.8)
    for(let i=0;i<12;i++){const x=cx+(rand()-.5)*w*.8,y=base-rand()*h*.6;line(g,[[x-11,y],[x+8,y-3],[x+14,y+2]],'#50594f',1.3)}
  }
  function branch(g,x,y,seed,fruit=false){
    const rand=A.rng(seed),height=fruit?67:27
    g.strokeStyle='#44483f';g.lineWidth=fruit?2:1.1;g.beginPath();g.moveTo(x,y);g.bezierCurveTo(x-9,y-height*.4,x-28,y-height*.6,x-19,y-height);g.stroke()
    for(let j=0;j<4;j++){
      const yy=y-height*(.25+j*.19),xx=x-5-j*4,dir=j%2?-1:1,tx=xx+dir*(8+rand()*12),ty=yy-5-rand()*9
      line(g,[[xx,yy],[tx,ty]],'#565c4e',.7)
      if(fruit){for(let n=0;n<2;n++){g.fillStyle=red;g.beginPath();g.ellipse(tx+n*4,ty+n*3,2.4,3,0,0,TAU);g.fill()}}
      else for(let n=0;n<5;n++){const a=n/5*TAU;g.fillStyle=red;g.beginPath();g.ellipse(tx+Math.cos(a)*1.5,ty+Math.sin(a)*1.5,1.5,1,a,0,TAU);g.fill()}
    }
  }
  function landmark(kind,res,seed){
    const sizes={gate:[500,320],trestle:[790,320],hall:[700,290],garden:[600,260],tower:[530,355],summit:[540,185]},[w,h]=sizes[kind]
    return A.sprite(w,h,res,g=>{
      g.lineCap='round';g.lineJoin='round'
      if(kind==='gate'){
        cliff(g,-170,0,190,290,seed);cliff(g,178,0,200,260,seed+2)
        g.fillStyle='#858b7e';g.beginPath();g.moveTo(-122,0);g.lineTo(-111,-220);g.lineTo(113,-230);g.lineTo(128,0);g.lineTo(77,0);g.lineTo(69,-153);g.quadraticCurveTo(0,-201,-70,-147);g.lineTo(-77,0);g.closePath();g.fill()
        for(let j=0;j<8;j++){const yy=-25-j*24;line(g,[[-119,yy],[-78,yy+4]],'#5a6457',1.2);line(g,[[78,yy+4],[123,yy]],'#5a6457',1.2)}
        line(g,[[-106,-219],[108,-228]],'#d4d6c1',2)
        g.fillStyle='#c3c7b1';g.fillRect(-24,-216,48,46);g.fillStyle='#3c463a';g.font='21px KaiTi,serif';g.textAlign='center';g.fillText('悬',0,-198);g.fillText('山',0,-176)
        branch(g,145,-23,7)
      }else if(kind==='trestle'){
        cliff(g,-240,0,310,305,seed);cliff(g,285,8,260,245,seed+1)
        g.fillStyle='#555b50';g.beginPath();g.moveTo(-360,-48);g.lineTo(362,-30);g.lineTo(367,-15);g.lineTo(-363,-31);g.closePath();g.fill()
        for(let x=-350;x<360;x+=30){line(g,[[x,-46],[x+8,-29]],'#aaa992',1);line(g,[[x,-45],[x,-88]],'#495148',2)}
        line(g,[[-356,-86],[-30,-77],[356,-70]],'#757b69',2)
        for(let x=-300;x<350;x+=120){line(g,[[x,-23],[x+35,38]],'#434c41',7);line(g,[[x-45,-27],[x+35,38]],'#656c59',4)}
        for(const x of [-300,320])line(g,[[x,0],[x,-230]],'#60695c',3)
        g.strokeStyle='#465344';g.lineWidth=1.5;g.beginPath();g.moveTo(-300,-226);g.quadraticCurveTo(0,-188,320,-231);g.stroke()
        branch(g,-132,-65,31,true)
      }else if(kind==='hall'){
        roof(g,0,-205,650)
        for(let x=-280;x<=280;x+=140){g.fillStyle='#676f60';g.fillRect(x-7,-197,14,198);line(g,[[x-4,-190],[x-3,-8]],'#aaa994',1);g.fillStyle='#868b78';g.fillRect(x-15,-9,30,9)}
        line(g,[[-287,-158],[287,-158]],'#656b5c',6)
        for(let x=-250;x<280;x+=70){line(g,[[x,-162],[x+35,-191],[x+70,-162]],'#7c826e',2)}
        for(let x=-210;x<=210;x+=140){g.strokeStyle='#929880';g.lineWidth=1;for(let j=0;j<4;j++)line(g,[[x-23+j*15,-151],[x-23+j*15,-37]],'#929880',.7)}
      }else if(kind==='garden'){
        A.drawSprite(g,A.pine(Math.max(.8,res*.8),41),-110,0,.8)
        g.fillStyle='#939786';g.fillRect(66,-30,108,10);g.fillRect(82,-20,12,20);g.fillRect(144,-20,12,20);line(g,[[66,-30],[174,-30]],'#ddd9c8',1.5)
        for(const x of [52,193]){g.fillStyle='#727b67';g.fillRect(x,-13,24,13);line(g,[[x,-13],[x+24,-13]],'#babea3',1)}
        branch(g,212,-2,12);branch(g,-237,-2,17)
      }else if(kind==='tower'){
        for(const x of [-132,132]){g.fillStyle='#4a5147';g.fillRect(x-10,-285,20,286);line(g,[[x-6,-281],[x-6,-4]],'#a8ad92',1.2)}
        roof(g,0,-281,348);roof(g,0,-178,408)
        line(g,[[-145,-237],[145,-237]],'#515b4b',9)
        for(let x=-115;x<=115;x+=46)line(g,[[x,-174],[x,-109]],'#747c65',2)
        for(let i=0;i<4;i++){g.fillStyle=i%2?'#929785':'#717a67';g.fillRect(-191-i*8,-10+i*5,382+i*16,6)}
        g.fillStyle='#c8c9b0';g.fillRect(-22,-278,44,37);g.fillStyle=ink;g.font='19px KaiTi,serif';g.textAlign='center';g.fillText('钟',0,-256)
      }else{
        g.fillStyle='#808779';g.beginPath();g.moveTo(-240,1);g.lineTo(-204,-31);g.lineTo(-145,-22);g.lineTo(175,-26);g.lineTo(248,3);g.closePath();g.fill()
        for(let j=0;j<9;j++)line(g,[[-211+j*51,1],[-197+j*51,-23]],'#545f51',.9)
        g.fillStyle='#6d7765';g.beginPath();g.moveTo(163,0);g.lineTo(158,-114);g.lineTo(180,-132);g.lineTo(204,-121);g.lineTo(211,0);g.fill()
        line(g,[[164,-110],[185,-124],[199,-115]],'#babfa6',1)
        g.fillStyle='#d4d5bb';g.font='18px KaiTi,serif';g.textAlign='center';g.fillText('止',184,-87);g.fillText('戈',185,-64)
        branch(g,-205,0,22)
      }
    },w/2,h-20)
  }
  function build(r,span){
    const out={mount:{}}
    for(const [key,f,base,alpha]of [['far',.06,248,.22],['mid',.16,272,.34],['near',.32,293,.4]])out[key]=A.mountains({width:span(f),top:-30,height:335,base,res:r(.35),seed:620+base,peakW:key==='far'?[35,95]:key==='mid'?[75,180]:[150,310],peakH:key==='far'?[100,220]:key==='mid'?[60,170]:[20,90],gap:[.55,1.05],round:1.5,detail:18,detailAmp:8,color:[78,83,76],alpha,depth:62,strokeCell:4,strokeDensity:.6,strokeLen:13,outline:.8,trees:.2,fadeH:55})
    for(const [i,k]of ['gate','trestle','hall','garden','tower','summit'].entries())out.mount[k]=landmark(k,r(.85),610+i*7)
    out.mountainPine=A.pine(r(.65),63)
    return out
  }
  const props=[['gate',950],['trestle',1800],['hall',2780],['garden',3690],['tower',4760],['summit',5770]]
  function background(g,a,cam){
    globalThis.XiabanInkScene.layer(g,cam,.48)
    for(let i=0;i<5;i++){g.save();g.translate(i*420+200,285);g.scale(.35,.35);A.drawSprite(g,a.mountainPine,0,0,.16);g.restore()}
  }
  function ground(g,a,cam){
    const left=cam.left-70,right=cam.left+cam.viewW+70
    for(const [k,x]of props)if(x+420>=left&&x-420<=right)A.drawSprite(g,a.mount[k],x,G+8,k==='gate'?.68:.61)
    const wood=(left+right)/2>1350&&(left+right)/2<2330
    g.fillStyle=wood?'#8c8b78':'#b8b9a7';g.fillRect(left,G,right-left,78)
    g.fillStyle='rgba(63,72,59,.17)';g.fillRect(left,G+33,right-left,45)
    for(let x=Math.floor(left/55)*55;x<right;x+=55){line(g,[[x,G+1],[x+6,G+19],[x-2,G+34]],'#858d79',.8);line(g,[[x+10,G+4],[x+43,G+4]],'#d7d5bf',.7)}
    for(let j=0;j<3;j++)line(g,[[left,G+10+j*12],[right,G+10+j*12]],wood?'#727963':'rgba(117,127,107,.35)',.7)
    line(g,[[left,G],[right,G]],'#414b3e',2)
    if(left<4110&&right>3350)for(const x of [3440,3990])branch(g,x,G+5,x)
  }
  function platform(p,res){return A.sprite(p.w+20,G-p.y+37,res,g=>{
    g.fillStyle='#7d8370';g.fillRect(0,-1,p.w,10);line(g,[[0,-1],[p.w,-1]],'#d3d1b8',1.3)
    for(const x of [9,p.w-12]){line(g,[[x,8],[x+3,G-p.y+5]],'#6b7560',6);line(g,[[p.w/2,9],[x+3,G-p.y+5]],'#959b83',3)}
    for(let x=0;x<=p.w;x+=p.w/4)line(g,[[x,0],[x,-22]],'#6f7a64',2)
    line(g,[[0,-20],[p.w,-20]],'#919a7e',1.5)
  },10,28)}
  function moving(g,a,cam,time,env){
    globalThis.XiabanInkScene.layer(g,cam,1)
    for(const x of [2490,3030])if(x>cam.left-30&&x<cam.left+cam.viewW+30){
      line(g,[[x,G],[x,G-136]],'#6c7563',2)
      const sw=Math.sin(time*3+x)*(2+(env.gust||0)*8),dir=env.windDir||1
      g.fillStyle='#b7bba3';g.beginPath();g.moveTo(x,G-133);g.lineTo(x+dir*28,G-130+sw);g.lineTo(x+dir*20,G-120+sw);g.lineTo(x,G-120);g.closePath();g.fill()
    }
    const bx=4760,by=G-163
    if(bx>cam.left-80&&bx<cam.left+cam.viewW+80){
      g.save();g.translate(bx,by-28);g.rotate(Math.sin(time*.9)*.035)
      line(g,[[0,-31],[0,0]],'#565d4a',3)
      g.fillStyle='#8e7953';g.strokeStyle='#514e3d';g.lineWidth=1.2;g.beginPath();g.moveTo(-14,0);g.quadraticCurveTo(-26,6,-29,43);g.lineTo(-35,49);g.quadraticCurveTo(0,57,35,49);g.lineTo(29,43);g.quadraticCurveTo(27,6,14,0);g.closePath();g.fill();g.stroke()
      for(let j=0;j<4;j++){g.beginPath();g.moveTo(-25-j,12+j*9);g.quadraticCurveTo(0,18+j*9,25+j,12+j*9);g.stroke()}
      for(const x of [-15,0,15])line(g,[[x,7],[x*1.4,43]],'#b09b6c',.7)
      line(g,[[0,37],[0,61]],'#4a4939',2.5);g.restore()
    }
  }
  globalThis.XiabanInkMountain={build,background,ground,platform,moving}
})()
