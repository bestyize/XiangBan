/* Canvas gestures emit bounded commands; only the trusted host writes project source. */
(() => {
  'use strict';
  addEventListener('DOMContentLoaded', () => {
    window.__OCD_CANVAS_EDITOR__=true;
    const source=window.__OCD_EDITOR_SOURCE__||document.body.cloneNode(true);
    const sourceMap=new Map([...source.querySelectorAll('[data-ocd-node]')].map(n=>[n.getAttribute('data-ocd-node'),n]));
    const revision=Number(location.pathname.split('/')[2]);
    const send=(type,data={})=>parent.postMessage({channel:'open-claude-design',token:new URLSearchParams(location.search).get('token'),...(Number.isInteger(revision)&&revision>0?{revision}:{}),type,...data},'*');
    let editing=true,writable=true,commenting=false,zoom=1,guides=true,selected=[],drag=null,textEdit=null,ignoreClick=false;
    const layer=document.createElement('div');layer.setAttribute('data-ocd-editor','');layer.style.cssText='position:fixed;inset:0;pointer-events:none;z-index:2147483645';document.body.append(layer);
    const own=el=>el?.closest?.('[data-ocd-editor],[data-ocd-pin-layer]');
    const id=el=>el?.getAttribute('data-ocd-node');
    const candidates=()=>[...document.querySelectorAll('[data-ocd-node]')].filter(el=>sourceMap.has(id(el))&&!['script','style','helmet','sc-for','sc-if','sc-else','sc-else-if'].includes(el.localName)&&!el.closest('[data-ocd-hidden="true"]'));
    const resolve=key=>candidates().find(el=>id(el)===key);
    const locked=el=>!!el.closest('[data-ocd-locked="true"]');
    const generated=el=>{const original=sourceMap.get(id(el));return !!el.closest('[data-ocd-imported]')||!original||!!original.closest('sc-for,dc-import');};
    const notice=message=>send('editor-notice',{message});
    function rect(el){
      let r=el.getBoundingClientRect();
      if(el.getAttribute('data-ocd-group')==='true')return union([...el.children].map(rect));
      return {x:r.left,y:r.top,w:r.width,h:r.height};
    }
    function union(boxes){if(!boxes.length)return {x:0,y:0,w:0,h:0};const x=Math.min(...boxes.map(r=>r.x)),y=Math.min(...boxes.map(r=>r.y));return {x,y,w:Math.max(...boxes.map(r=>r.x+r.w))-x,h:Math.max(...boxes.map(r=>r.y+r.h))-y};}
    const active=()=>selected.filter(el=>el.isConnected);
    function operands(){return active().flatMap(el=>el.getAttribute('data-ocd-group')==='true'?[...el.children]:[el]);}
    function info(el){
      const computed=getComputedStyle(el),styles={};for(const key of ['color','background-color','font-size','font-weight','text-align','padding','margin','border-radius','border-color','border-width','gap','width','height','opacity','display','flex-direction','align-items','justify-content'])styles[key]=computed.getPropertyValue(key);
      return {node:id(el),tag:el.tagName,text:el.children.length?'':el.textContent,path:el.localName+(el.id?'#'+el.id:''),quote:(el.textContent||'').trim().slice(0,160),styles,rect:rect(el),readonly:locked(el)||generated(el),group:el.hasAttribute('data-ocd-group')};
    }
    function announce(){const list=active();if(!list.length)send('select',{nodes:[],node:null});else send('select',{...info(list.at(-1)),nodes:list.map(info),commenting});draw();}
    function choose(els,add=false,individual=false){
      if(!individual){const keys=new Set(els.map(el=>el?.getAttribute('data-ocd-group')).filter(k=>k&&k!=='true'));if(keys.size)els=[...new Set([...els,...candidates().filter(el=>keys.has(el.getAttribute('data-ocd-group')))])];}
      if(textEdit)finishText(false);let list=add?[...active()]:[];
      for(const el of els){if(!el||own(el))continue;if(add&&list.includes(el))list=list.filter(n=>n!==el);else if(!list.includes(el))list.push(el);}
      selected=list.filter(el=>!list.some(other=>other!==el&&other.contains(el)));announce();
    }
    function draw(lines){
      layer.replaceChildren();if(!editing||textEdit)return;
      for(const el of active()){
        const r=rect(el),box=document.createElement('div');box.style.cssText=`position:fixed;left:${r.x}px;top:${r.y}px;width:${r.w}px;height:${r.h}px;border:${1.5/zoom}px solid #d94e60;box-sizing:border-box;pointer-events:none`;layer.append(box);
      }
      if(active().length&&!active().some(el=>locked(el)||generated(el))){const r=union(operands().map(rect)),s=8/zoom;
        for(const handle of ['nw','n','ne','e','se','s','sw','w']){const x=r.x+(handle.includes('e')?r.w:handle.includes('w')?0:r.w/2),y=r.y+(handle.includes('s')?r.h:handle.includes('n')?0:r.h/2);const h=document.createElement('button');h.type='button';h.dataset.handle=handle;h.setAttribute('aria-label','resize '+handle);h.style.cssText=`position:fixed;left:${x-s/2}px;top:${y-s/2}px;width:${s}px;height:${s}px;background:white;border:${1/zoom}px solid #d94e60;padding:0;pointer-events:auto;cursor:${handle}-resize;touch-action:none`;layer.append(h);}
      }
      if(lines&&guides){for(const [axis,v] of Object.entries(lines)){if(v==null)continue;const line=document.createElement('div');line.style.cssText=axis==='x'?`position:fixed;left:${v}px;top:0;height:100%;border-left:${1/zoom}px dashed #d94e60`:`position:fixed;left:0;top:${v}px;width:100%;border-top:${1/zoom}px dashed #d94e60`;layer.append(line);}}
    }
    function write(action){if(!writable){notice('正在保存，请稍候');return false;}if(active().some(el=>locked(el)||generated(el))){notice('此元素已锁定或由模板生成，请解锁或通过批注修改');return false;}send('element-edit',{action});return true;}
    function command(command){if(['delete','duplicate','cut','copy','group','ungroup'].includes(command)&&active().some(el=>locked(el)||generated(el))){notice('此元素已锁定或由模板生成，请通过图层或批注修改');return;}send('editor-command',{command,ids:active().map(id)});}
    function baseline(el){const s=getComputedStyle(el),tr=s.translate==='none'||!s.translate?'0px 0px':s.translate;const parts=tr.split(/\s+/);const x=/^-?[\d.]+px$/.test(parts[0])?parseFloat(parts[0]):0,y=/^-?[\d.]+px$/.test(parts[1])?parseFloat(parts[1]):0;return {el,id:id(el),box:rect(el),x,y,tx:parts[0]||'0px',ty:parts[1]||'0px',w:el.offsetWidth||rect(el).w,h:el.offsetHeight||rect(el).h,style:el.getAttribute('style')};}
    function translate(b,dx,dy){const shift=(old,n)=>/^-?[\d.]+px$/.test(old)?`${Math.round(parseFloat(old)+n)}px`:n?`calc(${old} + ${Math.round(n)}px)`:old;return {id:b.id,style:{translate:`${shift(b.tx,dx)} ${shift(b.ty,dy)}`}};}
    function reorder(el,dx,dy){
      const p=el.parentElement,cs=getComputedStyle(p),s=getComputedStyle(el);if(!['flex','inline-flex','grid','inline-grid'].includes(cs.display)||['absolute','fixed'].includes(s.position))return null;
      if(Math.abs(dx)+Math.abs(dy)<1)return null;const vertical=cs.display.includes('grid')?Math.abs(dy)>Math.abs(dx):cs.flexDirection.startsWith('column');const earlier=(vertical?dy:dx)<0;return {type:'reorder',ids:[id(el)],direction:earlier?'previous':'next'};
    }
    function nudge(dx,dy){const list=operands();if(!list.length)return;const order=list.length===1?reorder(list[0],dx,dy):null;if(order)write(order);else write({type:'style',patches:list.map(el=>translate(baseline(el),dx,dy))});}
    function snap(value,points){let best=null,delta=6/zoom;for(const p of points)if(Math.abs(p-value)<Math.abs(delta)){delta=p-value;best=p;}return best===null?{delta:0,line:null}:{delta,line:best};}
    function finishDrag(cancel=false){
      if(!drag)return;const d=drag;drag=null;for(const b of d.bases){if(b.style===null)b.el.removeAttribute('style');else b.el.setAttribute('style',b.style);}
      if(!cancel&&d.moved){ignoreClick=true;if(d.kind==='marquee'){const bounds={x:Math.min(d.startX,d.endX),y:Math.min(d.startY,d.endY),w:Math.abs(d.endX-d.startX),h:Math.abs(d.endY-d.startY)};choose([...(d.add?active():[]),...candidates().filter(el=>{const r=rect(el);return r.w>0&&r.h>0&&r.x>=bounds.x&&r.y>=bounds.y&&r.x+r.w<=bounds.x+bounds.w&&r.y+r.h<=bounds.y+bounds.h;})]);}
        else if(d.kind==='move'&&d.bases.length===1){const el=d.bases[0].el,p=el.parentElement,cs=getComputedStyle(p),s=getComputedStyle(el);if(['flex','inline-flex','grid','inline-grid'].includes(cs.display)&&!['absolute','fixed'].includes(s.position)){
          const others=[...p.children].filter(n=>n!==el&&id(n)&&sourceMap.has(id(n)));const vertical=cs.flexDirection.startsWith('column');const before=others.find(n=>{const r=rect(n);return vertical?d.endY<r.y+r.h/2:d.endX<r.x+r.w/2;});write({type:'reorder',ids:[id(el)],before:before?id(before):undefined});
        }else write({type:'style',patches:d.patches});}
        else if(d.kind!=='marquee')write({type:'style',patches:d.patches});
      }draw();
    }
    document.addEventListener('pointerdown',e=>{
      if(!editing||e.button!==0||textEdit)return;ignoreClick=false;
      const handle=e.target.closest?.('[data-handle]')?.dataset.handle;if(own(e.target)&&!handle)return;
      if(!writable){e.preventDefault();return;}
      let target=e.target.closest?.('[data-ocd-node]');while(target&&!sourceMap.has(id(target)))target=target.parentElement?.closest?.('[data-ocd-node]');
      if(commenting){if(target)choose([target]);return;}
      if(!handle&&target&&target.parentElement===document.querySelector('x-dc')&&target.children.length&&!selected.includes(target))target=null;
      if(!handle){if(target){if(!selected.includes(target)||e.shiftKey)choose([target],e.shiftKey);if(e.shiftKey){ignoreClick=true;e.preventDefault();return;}}else if(!e.shiftKey)choose([]);}
      const els=operands();if(els.some(el=>locked(el)||generated(el)))return;
      e.preventDefault();e.target.setPointerCapture?.(e.pointerId);
      const bases=els.map(baseline),bounds=union(bases.map(b=>b.box));const peers=bases.length===1?[...bases[0].el.parentElement.children].filter(n=>!els.includes(n)&&sourceMap.has(id(n))).slice(0,500):[];
      drag={kind:handle?'resize':target?'move':'marquee',handle,id:e.pointerId,startX:e.clientX,startY:e.clientY,endX:e.clientX,endY:e.clientY,bases,bounds,add:e.shiftKey,moved:false,patches:[],xs:[0,innerWidth/2,innerWidth,...peers.flatMap(n=>{const r=rect(n);return [r.x,r.x+r.w/2,r.x+r.w];})],ys:[0,innerHeight/2,innerHeight,...peers.flatMap(n=>{const r=rect(n);return [r.y,r.y+r.h/2,r.y+r.h];})]};
    },true);
    document.addEventListener('pointermove',e=>{
      if(!drag||drag.id!==e.pointerId)return;const d=drag;d.endX=e.clientX;d.endY=e.clientY;let dx=e.clientX-d.startX,dy=e.clientY-d.startY;if(Math.abs(dx)+Math.abs(dy)<3&&!d.moved)return;d.moved=true;
      if(d.kind==='marquee'){draw();const box=document.createElement('div');box.style.cssText=`position:fixed;left:${Math.min(d.startX,e.clientX)}px;top:${Math.min(d.startY,e.clientY)}px;width:${Math.abs(dx)}px;height:${Math.abs(dy)}px;border:${1/zoom}px solid #d94e60;background:#d94e6018`;layer.append(box);return;}
      let lines;
      if(d.kind==='move'){
        if(e.shiftKey){if(Math.abs(dx)>Math.abs(dy))dy=0;else dx=0;}
        if(guides&&!e.altKey){const sx=[0,.5,1].map(f=>snap(d.bounds.x+d.bounds.w*f+dx,d.xs)).filter(n=>n.line!==null).sort((a,b)=>Math.abs(a.delta)-Math.abs(b.delta))[0],sy=[0,.5,1].map(f=>snap(d.bounds.y+d.bounds.h*f+dy,d.ys)).filter(n=>n.line!==null).sort((a,b)=>Math.abs(a.delta)-Math.abs(b.delta))[0];if(sx)dx+=sx.delta;if(sy)dy+=sy.delta;lines={x:sx?.line,y:sy?.line};}
        d.patches=d.bases.map(b=>translate(b,dx,dy));
      }else{
        const h=d.handle;let dw=h.includes('e')?dx:h.includes('w')?-dx:0,dh=h.includes('s')?dy:h.includes('n')?-dy:0;
        if(e.altKey){dw*=2;dh*=2;}let w=Math.max(1,d.bounds.w+dw),height=Math.max(1,d.bounds.h+dh);if(e.shiftKey&&d.bounds.w&&d.bounds.h){const ratio=d.bounds.w/d.bounds.h;if(Math.abs(dw)>Math.abs(dh))height=w/ratio;else w=height*ratio;}
        const x=d.bounds.x+(e.altKey?(d.bounds.w-w)/2:h.includes('w')?d.bounds.w-w:0),y=d.bounds.y+(e.altKey?(d.bounds.h-height)/2:h.includes('n')?d.bounds.h-height:0),sx=w/(d.bounds.w||1),sy=height/(d.bounds.h||1);
        d.patches=d.bases.map(b=>({id:b.id,style:{...translate(b,x-d.bounds.x+(b.box.x-d.bounds.x)*(sx-1),y-d.bounds.y+(b.box.y-d.bounds.y)*(sy-1)).style,width:`${Math.max(1,Math.round(b.w*sx))}px`,height:`${Math.max(1,Math.round(b.h*sy))}px`,'box-sizing':'border-box'}}));
      }
      for(const patch of d.patches){const el=d.bases.find(b=>b.id===patch.id).el;for(const [k,v]of Object.entries(patch.style))el.style.setProperty(k,v);}draw(lines);
    });
    document.addEventListener('pointerup',()=>finishDrag());document.addEventListener('pointercancel',()=>finishDrag(true));
    document.addEventListener('dragstart',e=>{if(editing)e.preventDefault();});
    document.addEventListener('click',e=>{
      if(!editing||textEdit||own(e.target))return;e.preventDefault();e.stopImmediatePropagation();if(ignoreClick){ignoreClick=false;return;}
      const target=e.target.closest?.('[data-ocd-node]');if(target&&sourceMap.has(id(target)))choose([target],e.shiftKey);
    },true);
    function finishText(cancel){if(!textEdit)return;const t=textEdit;textEdit=null;const text=t.el.textContent??'';t.el.removeAttribute('contenteditable');t.el.removeEventListener('blur',t.blur);t.el.style.outline=t.outline;if(cancel)t.el.textContent=t.original;else if(text!==t.original)write({type:'text',id:id(t.el),text});draw();}
    function beginText(){
      const el=active().at(-1),original=el&&sourceMap.get(id(el));if(!el||!writable)return;
      if(el.namespaceURI!=='http://www.w3.org/1999/xhtml'||locked(el)||generated(el)||el.children.length||original?.textContent?.includes('{{')&&original.getAttribute('data-ocd-literal')!=='true'){notice('动态文字、组件或组合元素请通过批注修改');return;}
      textEdit={el,original:el.textContent,outline:el.style.outline,blur:()=>finishText(false)};el.setAttribute('contenteditable','plaintext-only');el.style.outline=`${1/zoom}px solid #d94e60`;el.focus();const range=document.createRange();range.selectNodeContents(el);const s=getSelection();s.removeAllRanges();s.addRange(range);el.addEventListener('blur',textEdit.blur,{once:true});draw();
    }
    document.addEventListener('dblclick',e=>{if(editing&&!commenting&&!textEdit&&!own(e.target)){e.preventDefault();const target=e.target.closest?.('[data-ocd-node]');if(target&&sourceMap.has(id(target))){choose([target],false,true);send('editor-command',{command:'showProperties'});beginText();}}},true);
    document.addEventListener('contextmenu',e=>{if(!editing||own(e.target))return;e.preventDefault();const target=e.target.closest?.('[data-ocd-node]');if(target&&!selected.includes(target))choose([target]);send('editor-context',{x:e.clientX,y:e.clientY});});
    function align(which){
      const bs=operands().map(baseline);if(!bs.length)return;const u=bs.length===1?rect(bs[0].el.parentElement||document.body):union(bs.map(b=>b.box));const patches=[];
      if(which==='distribute-x'||which==='distribute-y'){if(bs.length<3){notice('等距分布需要至少三个元素');return;}const x=which==='distribute-x';bs.sort((a,b)=>x?a.box.x-b.box.x:a.box.y-b.box.y);const span=x?u.w:u.h,total=bs.reduce((s,b)=>s+(x?b.box.w:b.box.h),0),gap=(span-total)/(bs.length-1);let pos=x?u.x:u.y;for(const b of bs){patches.push(translate(b,x?pos-b.box.x:0,x?0:pos-b.box.y));pos+=(x?b.box.w:b.box.h)+gap;}}
      else for(const b of bs){const r=b.box;const dx=which==='left'?u.x-r.x:which==='center-x'?u.x+(u.w-r.w)/2-r.x:which==='right'?u.x+u.w-r.w-r.x:0;const dy=which==='top'?u.y-r.y:which==='center-y'?u.y+(u.h-r.h)/2-r.y:which==='bottom'?u.y+u.h-r.h-r.y:0;patches.push(translate(b,dx,dy));}write({type:'style',patches});
    }
    function navigate(key,shift){const list=candidates();const current=active().at(-1);if(key==='Enter'&&shift){const p=current?.parentElement?.closest('[data-ocd-node]');if(p)choose([p]);return;}if(key==='Enter'){if(current?.children.length){const child=[...current.children].find(n=>sourceMap.has(id(n)));if(child)choose([child]);}else beginText();return;}const index=list.indexOf(current),next=key==='Home'?0:key==='End'?list.length-1:Math.max(0,Math.min(list.length-1,index+(['ArrowLeft','ArrowUp'].includes(key)||key==='Tab'&&shift?-1:1)));if(list[next])choose([list[next]]);}
    document.addEventListener('keydown',e=>{
      if(!editing||e.isComposing||e.keyCode===229)return;
      if(textEdit){if(e.key==='Escape'){e.preventDefault();finishText(true);}else if(e.key==='Tab'||e.key==='Enter'&&!e.shiftKey){e.preventDefault();finishText(false);}return;}
      if(e.target?.closest?.('input,textarea,select,[contenteditable]'))return;
      const mod=e.metaKey||e.ctrlKey,k=e.key.toLowerCase();
      if(e.key==='F6'){e.preventDefault();command(e.shiftKey?'focusPrevious':'focusNext');return;}
      if(mod&&['d','c','x','v','a','g','z','y','\\'].includes(k)){e.preventDefault();command(k==='d'?'duplicate':k==='c'?'copy':k==='x'?'cut':k==='v'?'paste':k==='a'?'selectAll':k==='g'?(e.shiftKey?'ungroup':'group'):k==='z'?(e.shiftKey?'redo':'undo'):k==='y'?'redo':'properties');return;}
      if(e.key==='Delete'||e.key==='Backspace'){e.preventDefault();command('delete');return;}
      if(e.key==='Escape'){e.preventDefault();finishDrag(true);choose([]);command('leaveArtboard');return;}
      if(e.shiftKey&&['h','v','g'].includes(k)){e.preventDefault();if(k==='g')command('guides');else{const patches=operands().map(el=>{const s=getComputedStyle(el).scale,parts=(s==='none'||!s?'1 1':s).split(/\s+/).map(Number);const x=parts[0]||1,y=parts[1]??x;return {id:id(el),style:{scale:`${k==='h'?-x:x} ${k==='v'?-y:y}`}};});write({type:'style',patches});}return;}
      if(['Tab','Home','End','Enter','ArrowLeft','ArrowRight','ArrowUp','ArrowDown'].includes(e.key)){e.preventDefault();if(e.altKey&&e.key.startsWith('Arrow')){const step=e.shiftKey?10:1;nudge(e.key==='ArrowLeft'?-step:e.key==='ArrowRight'?step:0,e.key==='ArrowUp'?-step:e.key==='ArrowDown'?step:0);}else navigate(e.key,e.shiftKey);return;}
      if([' ','+','=','-','0','1'].includes(e.key)){e.preventDefault();send('canvas-key',{key:e.key,down:true});}
    },true);
    document.addEventListener('keyup',e=>{if(editing&&e.key===' '&&!textEdit)send('canvas-key',{key:' ',down:false});});
    addEventListener('message',e=>{
      if(e.source!==parent||e.data?.channel!=='open-claude-design')return;const d=e.data;
      if(d.type==='mode'){if(textEdit&&(!d.editing||d.commenting))finishText(true);editing=!!d.editing;commenting=!!d.commenting;writable=d.writable!==false;zoom=Math.max(.1,Math.min(8,Number(d.zoom)||1));guides=d.guides!==false;if(!editing){finishDrag(true);selected=[];}draw();}
      if(d.type==='editor-selection'){selected=(Array.isArray(d.ids)?d.ids:[]).map(resolve).filter(Boolean);announce();}
      if(d.type==='editor-command'){if(d.command==='selectAll'){const container=active().at(-1)?.parentElement??document.querySelector('x-dc')??document.body;choose(candidates().filter(el=>el.parentElement===container));}else if(d.command==='enter'){const el=candidates()[0];if(el){choose([el]);el.setAttribute('tabindex','-1');el.focus();}}else if(d.command==='editText')beginText();else if(d.command?.startsWith('align:'))align(d.command.slice(6));else if(d.command==='clear')choose([]);}
    });
    addEventListener('resize',()=>draw());addEventListener('scroll',()=>draw(),true);addEventListener('ocd-rendered',()=>{selected=selected.map(el=>resolve(id(el))).filter(Boolean);draw();});
    send('editor-ready');
  });
})();
