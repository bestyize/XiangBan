export default function create({canvas, settings}) {
  const ctx=canvas.getContext('2d');
  let values=settings;
  return {
    update(next){values=next},
    render({time}){
      const w=canvas.width,h=canvas.height,t=time*Number(values.speed ?? 0.6),accent=values.accent ?? '#78b8a4';
      ctx.fillStyle='#edf8f1';ctx.fillRect(0,0,w,h);
      for(let i=0;i<5;i++){
        const x=w*(.5+.33*Math.sin(t*.12+i*1.7)),y=h*(.5+.3*Math.cos(t*.16+i));
        const r=Math.max(w,h)*.3,g=ctx.createRadialGradient(x,y,0,x,y,r);
        g.addColorStop(0,[accent+'80','#c5d7f480','#f7e4c480'][i%3]);g.addColorStop(1,'#edf8f100');
        ctx.fillStyle=g;ctx.fillRect(0,0,w,h);
      }
      ctx.save();ctx.translate(w*.5,h*.5);ctx.strokeStyle=accent+'55';ctx.lineWidth=Math.max(1,w/1400);
      for(let i=0;i<3;i++){ctx.beginPath();ctx.ellipse(0,0,w*(.28+i*.025),h*(.19+i*.012),Math.sin(t*.04+i)*.4,0,Math.PI*2);ctx.stroke()}
      ctx.restore();
    }
  }
}
