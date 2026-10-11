// Self-contained Canvas SDK scene, shared by desktop wallpaper and the interactive lantern pond.
export default function create({ canvas, dpr = 1, settings = {} }) {
  const ctx = canvas.getContext('2d');
  if (!ctx) throw new Error('无法绘制许愿灯池');
  let ratio = dpr, values = settings, backdrop = null, signature = '', wishes = [], ripples = [];
  const tau = Math.PI * 2;
  const seeded = i => { const v = Math.sin(i * 127.1 + 311.7) * 43758.5453; return v - Math.floor(v); };
  function readPositions() {
    if (typeof values._wishPositions !== 'string') return;
    try {
      const points = JSON.parse(values._wishPositions);
      if (Array.isArray(points)) wishes=points.slice(-24).filter(p=>p&&Number.isFinite(p.x)&&Number.isFinite(p.y)&&p.x>=.1&&p.x<=.9&&p.y>=.25&&p.y<=.85);
    } catch { wishes=[]; }
  }
  readPositions();
  function background(w, h) {
    const image = document.createElement('canvas'); image.width = Math.round(w); image.height = Math.round(h);
    const c = image.getContext('2d');
    const sky = c.createLinearGradient(0, 0, 0, h);
    sky.addColorStop(0, '#071c22'); sky.addColorStop(.33, '#16363a'); sky.addColorStop(.6, '#0c2a30'); sky.addColorStop(1, '#061b21');
    c.fillStyle = sky; c.fillRect(0, 0, w, h);
    const haze = c.createRadialGradient(w*.73,h*.23,0,w*.73,h*.23,w*.38);
    haze.addColorStop(0,'#aec8b41f'); haze.addColorStop(1,'#aec8b400'); c.fillStyle=haze;c.fillRect(0,0,w,h);
    for (let j=0;j<3;j++) {
      c.beginPath(); c.moveTo(0,h*.4);
      for(let x=0;x<=w+12;x+=12) c.lineTo(x,h*(.29+j*.025)-h*(.028*Math.sin(x/w*9+j*1.7)+.016*Math.sin(x/w*21+j)));
      c.lineTo(w,h*.43);c.lineTo(0,h*.43);c.closePath();c.fillStyle=['#203d3c','#183333','#112c30'][j];c.fill();
    }
    c.fillStyle='#eee6c2';c.beginPath();c.arc(w*.74,h*.18,Math.min(w,h)*.028,0,tau);c.fill();
    c.fillStyle='#203b3b';c.beginPath();c.arc(w*.75,h*.17,Math.min(w,h)*.026,0,tau);c.fill();
    for(let i=0;i<70;i++){ c.fillStyle=`rgba(211,222,205,${.12+seeded(i+5)*.35})`;c.beginPath();c.arc(w*seeded(i),h*.28*seeded(i+99),.35+seeded(i+48)*.8,0,tau);c.fill(); }
    // Quiet foliage frames the water; the centre stays open.
    for(let side=0;side<2;side++) {
      c.save(); if(side){c.translate(w,0);c.scale(-1,1);} c.strokeStyle='#061a20';c.lineWidth=5;
      c.beginPath();c.moveTo(-20,h*.08);c.bezierCurveTo(w*.025,h*.13,w*.065,h*.1,w*.12,h*.03);c.stroke();
      for(let i=0;i<18;i++){const x=w*.008+i*w*.006,y=h*(.09-.003*i);c.save();c.translate(x,y);c.rotate(-.8+seeded(i)*1.7);c.fillStyle='#092328';c.beginPath();c.ellipse(0,0,w*.012,h*.0035,0,0,tau);c.fill();c.restore();} c.restore();
    }
    for(let i=0;i<1600;i++){c.fillStyle=i%2?'#e7e1c403':'#00000008';c.fillRect(seeded(i+891)*w,seeded(i+1203)*h,1,1);}
    return image;
  }
  function lamp(x, y, size, time, fresh = false) {
    const flame = 1 + Math.sin(time*2.4+x)*.06;
    // Broken reflected light follows the water, fading with distance.
    for(let j=0;j<16;j++) {
      const length=size*(.65+j*.09)*(1+.15*Math.sin(time*1.3+j));
      ctx.fillStyle=`rgba(247,174,75,${(.16-j*.009)*flame})`;
      ctx.fillRect(x-length/2+Math.sin(time+j*.6)*size*.08,y+size*.25+j*size*.13,length,Math.max(.5,size*.028));
    }
    const glow=ctx.createRadialGradient(x,y,0,x,y,size*2.4);
    glow.addColorStop(0,`rgba(255,194,91,${.2*flame})`);glow.addColorStop(.3,'#ffb84b0b');glow.addColorStop(1,'#ffb84b00');
    ctx.fillStyle=glow;ctx.fillRect(x-size*2.4,y-size*2.4,size*4.8,size*4.8);
    ctx.save();ctx.translate(x,y);ctx.scale(size,size);
    ctx.fillStyle='#1a3e38';ctx.beginPath();ctx.ellipse(0,.18,1.04,.3,0,0,tau);ctx.fill();
    for(let i=0;i<7;i++) {
      const a=(i-3)*.3, px=Math.sin(a)*.78;
      ctx.save();ctx.translate(px,.07+Math.abs(a)*.05);ctx.rotate(a*.7);
      const petal=ctx.createLinearGradient(0,-.75,0,.3);petal.addColorStop(0,i%2?'#f3c9a0':'#e2a083');petal.addColorStop(.6,'#b16b57');petal.addColorStop(1,'#633e38');
      ctx.fillStyle=petal;ctx.beginPath();ctx.moveTo(0,.25);ctx.bezierCurveTo(-.48,-.02,-.35,-.57,0,-.82);ctx.bezierCurveTo(.37,-.54,.48,-.02,0,.25);ctx.fill();
      ctx.strokeStyle='#f0ca9f66';ctx.lineWidth=.016;ctx.stroke();ctx.restore();
    }
    const bowl=ctx.createLinearGradient(0,-.1,0,.37);bowl.addColorStop(0,'#f7ce85');bowl.addColorStop(1,'#995b3b');
    ctx.fillStyle=bowl;ctx.beginPath();ctx.ellipse(0,.08,.62,.25,0,0,tau);ctx.fill();
    ctx.fillStyle='#f7e2a6';ctx.beginPath();ctx.ellipse(0,-.01,.38,.12,0,0,tau);ctx.fill();
    ctx.fillStyle='#fff4c9';ctx.beginPath();ctx.moveTo(0,-.65*flame);ctx.bezierCurveTo(-.22,-.35,-.13,-.09,0,-.08);ctx.bezierCurveTo(.15,-.1,.19,-.36,0,-.65*flame);ctx.fill();
    if(fresh){ctx.strokeStyle='#f3daa180';ctx.lineWidth=.018;ctx.beginPath();ctx.ellipse(0,.23,1.35,.4,0,0,tau);ctx.stroke();}
    ctx.restore();
  }
  return {
    update(next) { values=next; readPositions(); },
    resize({ dpr: next }) { ratio=next || 1; signature=''; },
    setWishes(next) { wishes=next.slice(-24); },
    ripple(x,y,time) { ripples.push({x,y,time});ripples=ripples.slice(-8); },
    render({ time }) {
      const w=canvas.width/ratio,h=canvas.height/ratio,t=time*Number(values.speed ?? .6);
      ctx.setTransform(ratio,0,0,ratio,0,0);
      const key=`${w}:${h}`;if(signature!==key){backdrop=background(w,h);signature=key;}
      ctx.drawImage(backdrop,0,0,w,h);
      for(let i=0;i<46;i++) {
        const y=h*(.37+i*.0135),x=w*(.5+.34*Math.sin(i*2.1+t*.07)),length=w*(.02+.13*seeded(i+14));
        ctx.strokeStyle=`rgba(145,192,179,${.015+seeded(i)*.026})`;ctx.lineWidth=.6;
        ctx.beginPath();ctx.moveTo(x-length/2,y);ctx.quadraticCurveTo(x,y+Math.sin(t+i)*1.2,x+length/2,y);ctx.stroke();
      }
      // Small lotus leaves, kept low and at the edges.
      for(let i=0;i<9;i++){const x=w*seeded(i+110),y=h*(.71+.27*seeded(i+17));ctx.fillStyle='#25484088';ctx.beginPath();ctx.ellipse(x,y,w*(.018+seeded(i)*.015),h*.008,.2,0,tau);ctx.fill();}
      for(let i=0;i<9;i++) {
        const y=.4+seeded(i+50)*.43,x=.13+seeded(i+13)*.74;
        lamp(w*(x+Math.sin(t*.07+i)*.008),h*(y+Math.sin(t*.18+i)*.003),Math.min(w,h)*(.009+y*.028),t);
      }
      for(let i=0;i<wishes.length;i++){const v=wishes[i];lamp(w*(v.x+Math.sin(t*.12+i)*.004),h*(v.y+Math.cos(t*.17+i)*.003),Math.min(w,h)*(.017+v.y*.026),t,true);}
      for(const r of ripples){const age=time-r.time;if(age<0||age>3)continue;ctx.strokeStyle=`rgba(233,202,143,${.25*(1-age/3)})`;ctx.lineWidth=.7;ctx.beginPath();ctx.ellipse(w*r.x,h*r.y,20+age*38,6+age*10,0,0,tau);ctx.stroke();}
      ripples=ripples.filter(r=>time-r.time<3);
      for(let i=0;i<16;i++){const x=w*(seeded(i+412)+Math.sin(t*.11+i)*.005),y=h*(.36+seeded(i+611)*.45);ctx.fillStyle=`rgba(227,205,132,${.12+.22*(.5+.5*Math.sin(t+i*1.9))})`;ctx.beginPath();ctx.arc(x,y,1,0,tau);ctx.fill();}
    },
    destroy() { backdrop=null;wishes=[];ripples=[]; }
  };
}
