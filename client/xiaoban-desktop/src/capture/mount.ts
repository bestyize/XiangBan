import { invoke } from '../bridge.ts';
import markup from './editor.html?raw';
import './editor.css';
import { clamp, selectionBetween, resizeSelection, sourceRect, toolbarPosition, type Point, type Rect } from './geometry.ts';

type Tool = 'select' | 'rectangle' | 'arrow' | 'pen' | 'text' | 'mosaic';
type Mark = { type: Exclude<Tool,'select'>; points: Point[]; color: string; stroke: number; text?: string };
type Gesture = { mode: 'draw' | 'new' | 'move' | 'resize'; start: Point; original: Rect; edge?: string; id: number; history: Mark[] };
type Frame = { image: string; width: number; height: number };

export async function mountCapture() {
  document.body.className='capture-window';
  document.querySelector<HTMLElement>('#app')!.innerHTML=markup;
  const $ = <T extends HTMLElement=HTMLElement>(id:string) => document.getElementById(id) as T;
  const desktop=$<HTMLImageElement>('desktop'), selection=$('selection'), marks=$<HTMLCanvasElement>('marks'), capture=$('capture'), dock=$('dock');
  const ink=marks.getContext('2d');
  if(!ink) { await invoke('capture_finish',{image:null,error:'无法初始化截图画布，请重试。'}); return; }
  let rect:Rect={x:0,y:0,w:0,h:0}, tool:Tool='select', color='#f26baa', stroke=2;
  let history:Mark[]=[], draft:Mark|null=null, gesture:Gesture|null=null, textEntry:HTMLTextAreaElement|null=null;
  let busy=false, raf=0, disposed=false;
  let pixelated:HTMLCanvasElement|null=null;
  const viewport=()=>({x:innerWidth,y:innerHeight});
  const ratio=()=>desktop.naturalWidth/innerWidth;
  const error=(message:string)=>{ $('captureError').textContent=message; $('captureError').hidden=false; };
  const on = <K extends keyof HTMLElementEventMap>(node:HTMLElement,event:K,handler:(event:HTMLElementEventMap[K])=>void) => node.addEventListener(event,handler);
  function positionDock(){
    if(dock.hidden)return;
    const pos=toolbarPosition(rect,dock.offsetWidth,dock.offsetHeight,viewport());
    Object.assign(dock.style,{left:pos.left+'px',top:pos.top+'px'});
  }
  function normalized(e:PointerEvent):Point { return {x:clamp((e.clientX-rect.x)/rect.w,0,1),y:clamp((e.clientY-rect.y)/rect.h,0,1)}; }
  function renderMark(mark:Mark){
    const ctx=ink!;
    const points=mark.points.map(p=>({x:p.x*rect.w,y:p.y*rect.h})); const a=points[0],b=points[points.length-1];
    ctx.save();ctx.strokeStyle=mark.color;ctx.fillStyle=mark.color;ctx.lineWidth=mark.stroke;ctx.lineCap='round';ctx.lineJoin='round';
    if(mark.type==='rectangle') ctx.strokeRect(Math.min(a.x,b.x),Math.min(a.y,b.y),Math.abs(a.x-b.x),Math.abs(a.y-b.y));
    if(mark.type==='arrow'){
      const angle=Math.atan2(b.y-a.y,b.x-a.x),len=9+mark.stroke;
      ctx.beginPath();ctx.moveTo(a.x,a.y);ctx.lineTo(b.x,b.y);ctx.stroke();
      ctx.beginPath();ctx.moveTo(b.x,b.y);ctx.lineTo(b.x-len*Math.cos(angle-.4),b.y-len*Math.sin(angle-.4));ctx.lineTo(b.x-len*Math.cos(angle+.4),b.y-len*Math.sin(angle+.4));ctx.closePath();ctx.fill();
    }
    if(mark.type==='pen'){
      ctx.beginPath();ctx.moveTo(a.x,a.y);points.slice(1).forEach(p=>ctx.lineTo(p.x,p.y));
      if(points.every(p=>p.x===a.x&&p.y===a.y))ctx.lineTo(a.x+.1,a.y+.1);
      ctx.stroke();
    }
    if(mark.type==='text'){
      ctx.font='500 16px -apple-system,"PingFang SC","Microsoft YaHei",sans-serif';ctx.textBaseline='top';
      (mark.text||'').split('\n').forEach((line,i)=>ctx.fillText(line,a.x,a.y+i*22));
    }
    if(mark.type==='mosaic'&&pixelated){
      // Clip a continuous brush path, then redraw a coarse snapshot with smoothing disabled.
      const path=new Path2D(); const radius=15;
      for(let i=0;i<points.length;i++){
        const p=points[i];path.moveTo(p.x+radius,p.y);path.arc(p.x,p.y,radius,0,Math.PI*2);
        if(i){ const prev=points[i-1],angle=Math.atan2(p.y-prev.y,p.x-prev.x),dx=Math.sin(angle)*radius,dy=Math.cos(angle)*radius;
          path.moveTo(prev.x+dx,prev.y-dy);path.lineTo(p.x+dx,p.y-dy);path.lineTo(p.x-dx,p.y+dy);path.lineTo(prev.x-dx,prev.y+dy);path.closePath(); }
      }
      ctx.clip(path);ctx.imageSmoothingEnabled=false;
      ctx.drawImage(pixelated,-rect.x,-rect.y,innerWidth,innerHeight);
    }
    ctx.restore();
  }
  function render(){
    raf=0;ink!.clearRect(0,0,rect.w,rect.h);history.forEach(renderMark);if(draft)renderMark(draft);
    $<HTMLButtonElement>('undo').disabled=!history.length;
  }
  function schedule(){if(!raf)raf=requestAnimationFrame(render);}
  function syncRect(){
    Object.assign(selection.style,{left:rect.x+'px',top:rect.y+'px',width:rect.w+'px',height:rect.h+'px'});
    selection.hidden=rect.w<2||rect.h<2;$('shadeFull').hidden=!selection.hidden;selection.classList.toggle('near-top',rect.y<36);
    const src=sourceRect(rect,viewport(),{x:desktop.naturalWidth,y:desktop.naturalHeight});
    $('sizeText').textContent=`${Math.round(src.w)} × ${Math.round(src.h)} px`;
    // Limit annotation canvas memory on very large displays; exports are at most 2048px.
    const scale=Math.min(ratio()||1,2);
    const w=Math.max(1,Math.round(rect.w*scale)),h=Math.max(1,Math.round(rect.h*scale));
    if(marks.width!==w||marks.height!==h){marks.width=w;marks.height=h;}
    ink!.setTransform(w/(rect.w||1),0,0,h/(rect.h||1),0,0);schedule();positionDock();
  }
  function finishText(cancel=false){
    if(!textEntry)return;const input=textEntry;textEntry=null;
    if(input.value.trim()&&!cancel)history.push({type:'text',points:[{x:Number(input.dataset.x),y:Number(input.dataset.y)}],color:input.dataset.color!,stroke,text:input.value.trim()});
    input.remove();schedule();
  }
  function setTool(value:Tool){
    finishText();tool=value;
    document.querySelectorAll<HTMLButtonElement>('[data-tool]').forEach(b=>{b.classList.toggle('active',b.dataset.tool===value);b.setAttribute('aria-pressed',String(b.dataset.tool===value));});
    $('palette').hidden=value==='select'||value==='mosaic';selection.classList.toggle('drawing',value!=='select');
    $('guideText').textContent=value==='select'?'拖动选区可移动，拖动边角调整大小':value==='text'?'点击选区添加文字，Enter 确认':value==='mosaic'?'涂抹选区，遮住不想展示的内容':'在选区内拖动，添加标注';positionDock();
  }
  function addText(p:Point){
    finishText();const input=document.createElement('textarea');input.className='text-entry';input.placeholder='输入文字…';input.maxLength=2000;
    const x=clamp(p.x*rect.w,0,rect.w-110),y=clamp(p.y*rect.h,0,rect.h-34);
    input.dataset.x=String(x/rect.w);input.dataset.y=String(y/rect.h);input.dataset.color=color;
    Object.assign(input.style,{left:x+'px',top:y+'px',width:Math.min(180,rect.w)+'px',color});
    on(input,'pointerdown',e=>e.stopPropagation());
    on(input,'keydown',e=>{e.stopPropagation();if(e.isComposing)return;if(e.key==='Enter'&&!e.shiftKey){e.preventDefault();finishText();}if(e.key==='Escape'){e.preventDefault();finishText(true);}});
    on(input,'blur',()=>finishText());selection.append(input);textEntry=input;input.focus();
  }
  function reselect(){
    finishText(true);history=[];draft=null;rect={x:0,y:0,w:0,h:0};dock.hidden=true;setTool('select');syncRect();$('guideText').textContent='拖动鼠标框选截图区域';
  }
  on(capture,'pointerdown',e=>{
    if(busy||e.button!==0||dock.contains(e.target as Node)||textEntry?.contains(e.target as Node))return;
    finishText();$('captureError').hidden=true;
    const target=e.target as Element,handle=target.closest<HTMLElement>('[data-edge]'),inside=selection.contains(target);
    if(inside&&tool==='text'&&!handle){addText(normalized(e));return;}
    const base={start:{x:e.clientX,y:e.clientY},original:{...rect},id:e.pointerId,history:[...history]};
    if(handle) gesture={...base,mode:'resize',edge:handle.dataset.edge};
    else if(inside&&tool!=='select'){const p=normalized(e);draft={type:tool,points:[p,p],color,stroke};gesture={...base,mode:'draw'};}
    else if(inside)gesture={...base,mode:'move'};
    else {history=[];draft=null;gesture={...base,mode:'new'};rect={x:e.clientX,y:e.clientY,w:0,h:0};}
    dock.hidden=true;capture.setPointerCapture(e.pointerId);e.preventDefault();syncRect();
  });
  on(capture,'pointermove',e=>{
    const g=gesture;if(!g||e.pointerId!==g.id)return;
    const p={x:clamp(e.clientX,0,innerWidth),y:clamp(e.clientY,0,innerHeight)};
    if(g.mode==='draw'&&draft){
      const point=normalized(e);
      if(tool==='pen'||tool==='mosaic'){if(draft.points.length<12000)draft.points.push(point);}else draft.points[1]=point;
      schedule();return;
    }
    if(g.mode==='new')rect=selectionBetween(g.start,p);
    if(g.mode==='move')rect={...g.original,x:clamp(g.original.x+p.x-g.start.x,0,innerWidth-g.original.w),y:clamp(g.original.y+p.y-g.start.y,0,innerHeight-g.original.h)};
    if(g.mode==='resize')rect=resizeSelection(g.original,g.edge!,{x:p.x-g.start.x,y:p.y-g.start.y},viewport());
    syncRect();
  });
  function endGesture(e:PointerEvent,cancel=false){
    if(!gesture||gesture.id!==e.pointerId)return;
    if(cancel){rect=gesture.original;history=gesture.history;}
    else if(draft&&history.length<100)history.push(draft);
    draft=null;gesture=null;
    if(capture.hasPointerCapture(e.pointerId))capture.releasePointerCapture(e.pointerId);
    dock.hidden=rect.w<12||rect.h<12;syncRect();
  }
  on(capture,'pointerup',e=>endGesture(e));on(capture,'pointercancel',e=>endGesture(e,true));
  on(selection,'dblclick',e=>{if(tool==='select'&&!gesture){e.preventDefault();void complete();}});
  async function finish(image:string|null){
    if(busy)return;busy=true;capture.style.pointerEvents='none';
    try{await invoke('capture_finish',{image,error:null});}
    catch(e){busy=false;capture.style.pointerEvents='';error(String(e));}
  }
  async function complete(){
    if(busy||gesture||rect.w<12||rect.h<12)return;
    finishText();if(raf){cancelAnimationFrame(raf);raf=0;}render();
    try{
      const src=sourceRect(rect,viewport(),{x:desktop.naturalWidth,y:desktop.naturalHeight});
      const scale=Math.min(1,2048/Math.max(src.w,src.h));const canvas=document.createElement('canvas');
      canvas.width=Math.max(1,Math.round(src.w*scale));canvas.height=Math.max(1,Math.round(src.h*scale));
      const ctx=canvas.getContext('2d');if(!ctx)throw new Error('无法生成截图，请重试。');
      ctx.drawImage(desktop,src.x,src.y,src.w,src.h,0,0,canvas.width,canvas.height);
      ctx.drawImage(marks,0,0,canvas.width,canvas.height);
      let image='';for(const quality of [.9,.8,.65,.5]){image=canvas.toDataURL('image/jpeg',quality);if(image.length<3*1024*1024/3*4)break;}
      if(image.length>3*1024*1024/3*4)throw new Error('截图太大，请缩小选区后重试。');
      await finish(image);
    }catch(e){error(e instanceof Error?e.message:String(e));}
  }
  document.querySelectorAll<HTMLButtonElement>('[data-tool]').forEach(b=>on(b,'click',()=>setTool(b.dataset.tool as Tool)));
  for(const key of ['color','width'] as const)document.querySelectorAll<HTMLButtonElement>(`[data-${key}]`).forEach(b=>on(b,'click',()=>{
    if(key==='color')color=b.dataset.color!;else stroke=Number(b.dataset.width);
    document.querySelectorAll<HTMLButtonElement>(`[data-${key}]`).forEach(other=>{other.classList.toggle('active',other===b);other.setAttribute('aria-pressed',String(other===b));});
  }));
  on($('undo'),'click',()=>{finishText();history.pop();schedule();});on($('reselect'),'click',reselect);
  on($('cancel'),'click',()=>{finishText(true);void finish(null);});on($('confirm'),'click',()=>void complete());
  const keyboard=(e:KeyboardEvent)=>{
    if(textEntry||e.isComposing||busy)return;
    if(e.key==='Escape'){e.preventDefault();void finish(null);}
    if(e.key==='Enter'&&!(e.target instanceof HTMLButtonElement)){e.preventDefault();void complete();}
    if((e.metaKey||e.ctrlKey)&&e.key.toLowerCase()==='z'){e.preventDefault();history.pop();schedule();}
  };
  document.addEventListener('keydown',keyboard);document.addEventListener('contextmenu',e=>e.preventDefault());
  const resize=()=>{rect.w=Math.min(rect.w,innerWidth);rect.h=Math.min(rect.h,innerHeight);rect.x=clamp(rect.x,0,innerWidth-rect.w);rect.y=clamp(rect.y,0,innerHeight-rect.h);syncRect();};
  window.addEventListener('resize',resize);
  window.addEventListener('beforeunload',()=>{disposed=true;cancelAnimationFrame(raf);desktop.src='';pixelated=null;history=[];document.removeEventListener('keydown',keyboard);window.removeEventListener('resize',resize);},{once:true});
  try{
    const frame=await invoke<Frame>('capture_frame');
    await new Promise<void>((resolve,reject)=>{desktop.onload=()=>resolve();desktop.onerror=()=>reject(new Error('无法读取屏幕画面'));desktop.src=frame.image;});
    if(disposed)return;
    pixelated=document.createElement('canvas');pixelated.width=Math.max(1,Math.ceil(innerWidth/12));pixelated.height=Math.max(1,Math.ceil(innerHeight/12));
    pixelated.getContext('2d')!.drawImage(desktop,0,0,pixelated.width,pixelated.height);
    reselect();await invoke('capture_show');
  }catch(e){await invoke('capture_finish',{image:null,error:'截图窗口无法打开：'+String(e)}).catch(()=>{});}
}
