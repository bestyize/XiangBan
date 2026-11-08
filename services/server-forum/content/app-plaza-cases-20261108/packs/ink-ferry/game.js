(()=>{
const $=s=>document.querySelector(s), canvas=$('#board'),ctx=canvas.getContext('2d'), overlay=$('#overlay');
const xb = globalThis.xb;
const report = error => console.warn('墨渡运行时：', error);
let state='intro',score=0,best=0,combo=0,muted=false,dirty=false,charge=0,gap=150,radius=36,wind=0,flight=null,particles=[],clock=0,last=0,audio=null,pressed=false,feedback='',feedbackTime=0;
function save(){dirty=true;xb?.storage.set('save',{best,muted}).catch(report)}

async function restore() {
  let saved = await xb.storage.get('save');
  if (dirty) return;
  if (saved === null && xb.info.legacySave !== null) {
    saved = xb.info.legacySave;
    await xb.storage.set('save', saved);
  }
  if (dirty || !saved || typeof saved !== 'object') return;
  best = Number.isFinite(saved.best) ? Math.max(0, Math.floor(saved.best)) : 0;
  muted = saved.muted === true;
  ui();
}
function ui(){$('#score').textContent=score;$('#best').textContent=best;$('#combo').textContent=combo;$('#wind').textContent=wind===0?'无':wind>0?'→ '+Math.abs(wind):'← '+Math.abs(wind);$('#sound').textContent=muted?'声音 关':'声音 开';$('#pause').textContent=state==='paused'?'继续':'暂停';$('#fill').style.width=(Math.min(charge,1.6)/1.6*100)+'%';$('#mark').style.left=Math.max(0,Math.min(100,((gap-70)/120)/1.6*100))+'%'}
function tone(freq,duration,type='sine'){if(muted)return;try{audio=audio||new (window.AudioContext||window.webkitAudioContext)();if(audio.state==='suspended')audio.resume();const o=audio.createOscillator(),g=audio.createGain(),t=audio.currentTime;o.type=type;o.frequency.setValueAtTime(freq,t);o.frequency.exponentialRampToValueAtTime(freq*.7,t+duration);g.gain.setValueAtTime(.0001,t);g.gain.exponentialRampToValueAtTime(.06,t+.01);g.gain.exponentialRampToValueAtTime(.0001,t+duration);o.connect(g);g.connect(audio.destination);o.start(t);o.stop(t+duration)}catch(e){}}
function modal(title,msg,button,sub){$('#title').textContent=title;$('#message').innerHTML=msg;$('#action').textContent=button;$('#sub').textContent=sub;overlay.hidden=false}
function next(){gap=115+Math.random()*Math.min(90,38+score*.8);radius=Math.max(21,38-score*.12);wind=score<50?0:Math.floor(Math.random()*5)-2;charge=0;ui()}
function start(){document.activeElement.blur();state='playing';score=combo=0;particles=[];flight=null;overlay.hidden=true;next();xb?.game.start().catch(report);ui()}
function finish(){state='ended';pressed=false;if(score>best){best=score;save()}xb?.game.result({score}).catch(report);modal('此行暂歇',`本局 ${score} 分<br>最高 ${best} 分`,'再渡一次','山水有相逢');ui()}
function pause(){if(state==='playing'){state='paused';pressed=false;charge=0;modal('舟泊片刻','歇一歇，准备好了再继续。','继续渡江','水声未远')}else if(state==='paused'){state='playing';overlay.hidden=true}ui()}
function begin(e){if(state!=='playing'||flight||pressed)return;if(e)e.preventDefault();pressed=true;charge=0;tone(240,.08)}
function release(){if(!pressed)return;pressed=false;if(state!=='playing'||flight)return;const distance=70+Math.min(charge,1.6)*120;flight={t:0,duration:.7,distance,dx:wind*5};charge=0;tone(175,.18,'triangle')}
function land(){const error=Math.hypot(flight.distance-gap,flight.dx);flight=null;if(error>radius+5){feedback='落水';feedbackTime=1;tone(90,.35);finish();return}const perfect=error<radius*.32;combo=perfect?combo+1:0;const points=perfect?10+Math.min(combo,10)*2:5;score+=points;feedback=perfect?`正中 · +${points}`:`落稳 · +${points}`;feedbackTime=1.1;for(let i=0;i<10;i++)particles.push({x:180,y:390-gap,vx:(Math.random()-.5)*90,vy:(Math.random()-.7)*65,life:.6});tone(perfect?600+Math.min(combo,8)*35:360,.18,'triangle');xb?.game.progress({score}).catch(report);next()}
function resize(){const r=canvas.getBoundingClientRect(),d=Math.min(devicePixelRatio||1,3);canvas.width=Math.round(r.width*d);canvas.height=Math.round(r.height*d)}addEventListener('resize',resize);resize();
function draw(){ctx.setTransform(canvas.width/360,0,0,canvas.height/470,0,0);ctx.clearRect(0,0,360,470);ctx.fillStyle='#e9e8d9';ctx.fillRect(0,0,360,470);
// Distant ink wash and ripples
ctx.fillStyle='#a9b9a833';ctx.beginPath();ctx.moveTo(0,158);ctx.lineTo(65,70);ctx.lineTo(105,120);ctx.lineTo(176,51);ctx.lineTo(255,130);ctx.lineTo(308,83);ctx.lineTo(360,150);ctx.lineTo(360,210);ctx.lineTo(0,210);ctx.fill();ctx.strokeStyle='#748d7d55';ctx.lineWidth=1;for(let i=0;i<11;i++){let y=135+i*32;ctx.beginPath();ctx.ellipse((i*93+clock*4)%460-50,y,35+i%3*18,2,0,0,Math.PI*2);ctx.stroke()}ctx.strokeStyle='#546b60aa';let bx=(clock*14)%480-60;ctx.beginPath();ctx.moveTo(bx,55);ctx.quadraticCurveTo(bx+5,50,bx+10,55);ctx.quadraticCurveTo(bx+15,50,bx+20,55);ctx.stroke();
function island(y,r,col){ctx.fillStyle=col;ctx.beginPath();ctx.ellipse(180,y,r,r*.34,0,0,7);ctx.fill();ctx.strokeStyle='#536d5e';ctx.beginPath();ctx.ellipse(180,y-1,r*.77,r*.2,0,.1,3);ctx.stroke()}island(390,41,'#8ba28a');island(390-gap,radius,'#6e8b75');ctx.fillStyle='#9d4938';ctx.beginPath();ctx.arc(180,390-gap,2.5,0,7);ctx.fill();
let x=180,y=390;if(flight){let t=Math.min(1,flight.t/flight.duration);x+=flight.dx*t;y-=flight.distance*t+Math.sin(t*Math.PI)*40}ctx.fillStyle='#293e37';ctx.beginPath();ctx.moveTo(x-16,y-5);ctx.quadraticCurveTo(x,y+10,x+16,y-5);ctx.lineTo(x+11,y+4);ctx.quadraticCurveTo(x,y+13,x-11,y+4);ctx.closePath();ctx.fill();ctx.strokeStyle='#293e37';ctx.beginPath();ctx.moveTo(x,y-5);ctx.lineTo(x+1,y-25);ctx.lineTo(x+10,y-8);ctx.stroke();ctx.fillStyle='#eee8d2';ctx.beginPath();ctx.moveTo(x+2,y-23);ctx.lineTo(x+2,y-8);ctx.lineTo(x+9,y-8);ctx.fill();
for(const p of particles){ctx.fillStyle=`rgba(104,142,119,${Math.max(0,p.life)})`;ctx.beginPath();ctx.arc(p.x,p.y,2.5,0,7);ctx.fill()}if(feedbackTime>0){ctx.fillStyle='#a44735';ctx.textAlign='center';ctx.font='16px serif';ctx.fillText(feedback,180,100)}if(pressed){ctx.fillStyle='#394f43';ctx.textAlign='center';ctx.font='14px serif';ctx.fillText('蓄力中 · 松手起航',180,445)}}
function frame(t){let dt=Math.min((t-last)/1000||0,1/30);last=t;clock+=dt;if(state==='playing'){if(pressed){charge=Math.min(1.6,charge+dt);ui()}if(flight){flight.t+=dt;if(flight.t>=flight.duration)land()}feedbackTime=Math.max(0,feedbackTime-dt);for(const p of particles){p.x+=p.vx*dt;p.y+=p.vy*dt;p.life-=dt}particles=particles.filter(p=>p.life>0)}draw();requestAnimationFrame(frame)}requestAnimationFrame(frame);
canvas.addEventListener('pointerdown',e=>{canvas.setPointerCapture(e.pointerId);begin(e)});canvas.addEventListener('pointerup',release);canvas.addEventListener('pointercancel',()=>{pressed=false;charge=0;ui()});window.addEventListener('keydown',e=>{if(e.code==='Space'&&!e.repeat&&!['BUTTON'].includes(document.activeElement.tagName)){e.preventDefault();begin()}});window.addEventListener('keyup',e=>{if(e.code==='Space'){e.preventDefault();release()}});window.addEventListener('blur',()=>{if(state==='playing')pause()});document.addEventListener('visibilitychange',()=>{if(document.hidden&&state==='playing')pause()});$('#action').onclick=()=>state==='paused'?pause():start();$('#pause').onclick=pause;$('#sound').onclick=()=>{muted=!muted;save();ui();tone(350,.09)};ui();

if (xb) {
  xb.on('hide', () => {
    if (state === 'playing') pause();
    audio?.suspend().catch(report);
  });
  restore().catch(report).then(() => xb.ready()).catch(report);
}
})();
