import svgpath from 'svgpath';

const excluded='script,style,link,meta,template,[data-ocd-editor],[data-ocd-pin-layer]';
const clamp=n=>Math.max(0,Math.min(1,n));
const colorCache=new Map();
const fontCache=new Map();
function cjkFont(style){
 const serif=/\bserif\b/.test(style.fontFamily)&&!/sans-serif/.test(style.fontFamily),key=serif?'serif':'sans';if(fontCache.has(key))return fontCache.get(key);
 const c=document.createElement('canvas'),ctx=c.getContext('2d'),sample='mmmmmmmmWWWW12345';ctx.font='20px monospace';const fallback=ctx.measureText(sample).width;
 const candidates=serif?['Songti SC','SimSun','Noto Serif CJK SC','Noto Serif SC']:['PingFang SC','Microsoft YaHei','Noto Sans CJK SC','Noto Sans SC'];
 const found=candidates.find(f=>{ctx.font=`20px "${f}",monospace`;return Math.abs(ctx.measureText(sample).width-fallback)>.1;})??style.fontFamily.split(',')[0].trim().replace(/["']/g,'');fontCache.set(key,found);return found;
}
export function splitCss(value){
 const parts=[];let depth=0,start=0,quote='';
 for(let i=0;i<value.length;i++){const c=value[i];if(quote){if(c===quote&&value[i-1]!=='\\')quote='';}else if(c==='"'||c==="'")quote=c;else if(c==='(')depth++;else if(c===')')depth--;else if(c===','&&!depth){parts.push(value.slice(start,i).trim());start=i+1;}}
 parts.push(value.slice(start).trim());return parts;
}
export function readColor(value){
 if(!value||value==='none'||value==='transparent')return {hex:'000000',alpha:0};
 if(colorCache.has(value))return {...colorCache.get(value)};
 const c=document.createElement('canvas');c.width=c.height=1;const ctx=c.getContext('2d');
 ctx.fillStyle=value;ctx.fillRect(0,0,1,1);const p=ctx.getImageData(0,0,1,1).data;
 const result={hex:[...p.slice(0,3)].map(n=>n.toString(16).padStart(2,'0')).join(''),alpha:p[3]/255};if(colorCache.size<2000)colorCache.set(value,result);return {...result};
}
export function gradient(value,w,h){
 const match=/^(linear|radial)-gradient\((.*)\)$/s.exec(value);if(!match)return;
 const parts=splitCss(match[2]);let angle=180;
 if(match[1]==='linear'&&/^(to\s|[-+.\d]+(?:deg|turn|rad))/.test(parts[0])){
  const d=parts.shift();const directions={'to top':0,'to right':90,'to bottom':180,'to left':270,'to bottom right':135,'to right bottom':135,'to top right':45,'to right top':45,'to bottom left':225,'to left bottom':225,'to top left':315,'to left top':315};
  angle=directions[d]??parseFloat(d)*(d.endsWith('turn')?360:d.endsWith('rad')?180/Math.PI:1);
 }else if(match[1]==='radial'&&!/^(#|rgb|hsl|okl|color\(|transparent\b|[a-z]+\s+[\d.])/.test(parts[0]))parts.shift();
 const length=Math.abs(w*Math.sin(angle*Math.PI/180))+Math.abs(h*Math.cos(angle*Math.PI/180));
 const stops=parts.map(p=>{const m=/^(.*?)(?:\s+([-+.\d]+)(%|px))?$/.exec(p);return {color:readColor(m[1]),offset:m[2]===undefined?null:clamp(Number(m[2])/(m[3]==='%'?100:length||1))};});
 if(stops.length<2)return;
 stops[0].offset??=0;stops[stops.length-1].offset??=1;
 for(let i=1;i<stops.length;i++){if(stops[i].offset!==null){stops[i].offset=Math.max(stops[i-1].offset??0,stops[i].offset);continue;}let end=i;while(stops[end].offset===null)end++;for(let j=i;j<end;j++)stops[j].offset=stops[i-1].offset+(stops[end].offset-stops[i-1].offset)*(j-i+1)/(end-i+1);i=end-1;}
 return {type:match[1],angle,stops};
}
function shadows(value){
 if(!value||value==='none')return [];
 return splitCss(value).map(v=>{const m=/(rgba?\([^)]+\)|hsla?\([^)]+\)|#[\da-f]+|color\([^)]+\))/i.exec(v),nums=(m?v.replace(m[0],''):v).match(/[-+.\d]+(?=px)/g)?.map(Number)??[];
 return {x:nums[0]??0,y:nums[1]??0,blur:Math.max(0,nums[2]??0),spread:nums[3]??0,inset:v.includes('inset'),color:readColor(m?.[0]??'black')};}).slice(0,20);
}
function roundedPath(w,h,radii){
 const [a,b,c,d]=radii.map(r=>Math.min(r,w/2,h/2)),k=.55228475;
 return `M${a} 0 L${w-b} 0 C${w-b+b*k} 0 ${w} ${b-b*k} ${w} ${b} L${w} ${h-c} C${w} ${h-c+c*k} ${w-c+c*k} ${h} ${w-c} ${h} L${d} ${h} C${d-d*k} ${h} 0 ${h-d+d*k} 0 ${h-d} L0 ${a} C0 ${a-a*k} ${a-a*k} 0 ${a} 0 Z`;
}
export async function extractScene(width,height,raster){
 const nodes=[],warnings=new Set();let count=0,textCount=0;
 const warn=s=>{if(warnings.size<200)warnings.add(s.slice(0,500));};
 const base={version:1,width,height,nodes,warnings:[]};
 // Replace generated pseudo content with temporary DOM twins, so it participates
 // in the exact same browser layout and becomes editable text/geometry.
 const pseudoCleanup=[],pseudoStyle=document.createElement('style');
 pseudoStyle.textContent='[data-xb-export-pseudo]::before,[data-xb-export-pseudo]::after{content:none!important}';document.head.append(pseudoStyle);
 for(const el of document.body.querySelectorAll('*')){
  if(el.matches(excluded))continue;
  const twins=[];
  for(const which of ['::before','::after']){
   const ps=getComputedStyle(el,which),content=ps.content;
   if(!content||content==='none'||content==='normal'||ps.display==='none')continue;
   if(!/^(["']).*\1$/s.test(content)){warn(`${styleName(el)}：${which} 的动态内容未转换`);continue;}
   const twin=document.createElement('span');for(const key of ps)twin.style.setProperty(key,ps.getPropertyValue(key));
   twin.style.setProperty('content','normal');twin.textContent=content.slice(1,-1).replace(/\\([\da-f]{1,6})\s?/gi,(_,code)=>String.fromCodePoint(parseInt(code,16))).replace(/\\(["'\\])/g,'$1');
   twin.setAttribute('data-xb-pseudo-twin',which);twins.push({twin,which});
  }
  if(twins.length){const old=el.getAttribute('data-xb-export-pseudo');el.setAttribute('data-xb-export-pseudo','');for(const {twin,which} of twins){if(which==='::before')el.prepend(twin);else el.append(twin);}pseudoCleanup.push(()=>{for(const {twin} of twins)twin.remove();if(old===null)el.removeAttribute('data-xb-export-pseudo');else el.setAttribute('data-xb-export-pseudo',old);});}
 }
 // Artboard coordinates start at the document origin, not the body's margin box.
 const origin={x:-window.scrollX,y:-window.scrollY};
 const box=el=>{const r=el.getBoundingClientRect();return {x:r.x-origin.x,y:r.y-origin.y,w:r.width,h:r.height};};
 const push=n=>{if(nodes.length>=20000)throw Error('画板超过 20000 个导出元素');if(n.w>0&&n.h>0)nodes.push(n);};
 const visible=b=>b.x+b.w>0&&b.y+b.h>0&&b.x<width&&b.y<height;
 async function bitmap(el,b,opacity,name,reason){
  const image=await raster(el,Math.max(1,Math.ceil(b.w)),Math.max(1,Math.ceil(b.h)));
  push({kind:'image',name,x:b.x,y:b.y,w:b.w,h:b.h,opacity:opacity/(Number(getComputedStyle(el).opacity)||1),image});warn(`${name}：${reason}，已保留为图片`);
 }
 function fills(style,b){const out=[],solid=readColor(style.backgroundColor);if(solid.alpha)out.push({type:'solid',color:solid});for(const layer of splitCss(style.backgroundImage??'none').reverse()){const f=gradient(layer,b.w,b.h);if(f)out.push(f);}return out;}
 function styleName(el){return (el.getAttribute('aria-label')||el.id||el.getAttribute('data-name')||el.tagName.toLowerCase()).slice(0,180);}
 async function svg(el,opacity){
  const root=box(el);if(!root.w||!root.h)return;
  const svgOpacity=child=>{let value=opacity;for(let n=child;n&&n!==el;n=n.parentElement)value*=Number(getComputedStyle(n).opacity);return value;};
  for(const child of el.querySelectorAll('path,rect,circle,ellipse,line,polyline,polygon,text,use,image,foreignObject,g')){
   if(child.closest('defs,clipPath,mask,symbol'))continue;
   const st=getComputedStyle(child);if(st.display==='none'||st.visibility==='hidden')continue;
   if(['use','image','foreignObject'].includes(child.localName)||st.filter!=='none'||child.hasAttribute('clip-path')||child.hasAttribute('mask')||st.markerStart!=='none'||st.markerEnd!=='none'||st.markerMid!=='none'){await bitmap(el,root,opacity,styleName(el),'包含暂不能转换的 SVG 引用、标记或蒙版');return;}
  }
  for(const child of el.querySelectorAll('path,rect,circle,ellipse,line,polyline,polygon,text')){
   if(child.closest('defs,clipPath,mask,symbol'))continue;
   const st=getComputedStyle(child);if(st.display==='none'||st.visibility==='hidden')continue;
   if(child.localName==='text'){const elText=child;
   const walker=document.createTreeWalker(elText,NodeFilter.SHOW_TEXT);let node;
   while((node=walker.nextNode())){
    const st=getComputedStyle(node.parentElement);if(st.visibility==='hidden'||st.display==='none')continue;
    const textStyle=new Proxy(st,{get(target,key){return key==='color'?target.fill:Reflect.get(target,key,target);}});
    const start=nodes.length;text(node,textStyle,svgOpacity(node.parentElement),styleName(elText),{x:0,y:0,w:width,h:height});
    // SVG text uses fill-opacity in addition to the element's opacity.
    const matrix=elText.getScreenCTM();for(const n of nodes.slice(start)){n.opacity*=Number(st.fillOpacity);if(matrix){n.fontSize*=Math.hypot(matrix.c,matrix.d);n.letterSpacing*=Math.hypot(matrix.a,matrix.b);}}
   }
    continue;
   }
   let d=child.getAttribute('d');const n=k=>Number(child.getAttribute(k)||0);
   if(child.localName==='rect'){const w=n('width'),h=n('height'),rx=n('rx')||n('ry');d=svgpath(roundedPath(w,h,[rx,rx,rx,rx])).translate(n('x'),n('y')).toString();}
   if(child.localName==='circle'||child.localName==='ellipse'){const rx=n('r')||n('rx'),ry=n('r')||n('ry');d=`M${n('cx')-rx} ${n('cy')} a${rx} ${ry} 0 1 0 ${2*rx} 0 a${rx} ${ry} 0 1 0 ${-2*rx} 0Z`;}
   if(child.localName==='line')d=`M${n('x1')} ${n('y1')} L${n('x2')} ${n('y2')}`;
   if(['polyline','polygon'].includes(child.localName))d='M'+(child.getAttribute('points')||'')+(child.localName==='polygon'?'Z':'');
   if(!d)continue;
   const m=child.getScreenCTM();if(!m)continue;
   d=svgpath(d).matrix([m.a,m.b,m.c,m.d,m.e-origin.x,m.f-origin.y]).abs().unshort().unarc().round(4).toString();
   // Normalize H/V and quadratic commands using svgpath's iteration API.
   const normalized=svgpath(d);normalized.iterate((seg,i,x,y)=>{if(seg[0]==='H')return [['L',seg[1],y]];if(seg[0]==='V')return [['L',x,seg[1]]];});d=normalized.toString();
   let fill;const ref=/url\(["']?#([^"')]+)["']?\)/.exec(st.fill);
   if(ref){const g=el.querySelector(`[id="${CSS.escape(ref[1])}"]`);if(g){const stops=[...g.querySelectorAll('stop')].map(s=>{const cs=getComputedStyle(s),color=readColor(cs.stopColor);color.alpha*=Number(cs.stopOpacity);return {offset:clamp(parseFloat(s.getAttribute('offset')||'0')/(s.getAttribute('offset')?.endsWith('%')?100:1)),color};});if(stops.length>=2){const val=k=>parseFloat(g.getAttribute(k)??({x1:'0',y1:'0',x2:'100',y2:'0'}[k]));fill={type:g.localName==='radialGradient'?'radial':'linear',angle:90-Math.atan2(val('y1')-val('y2'),val('x2')-val('x1'))*180/Math.PI,stops};}}}
   else if(st.fill!=='none'){const c=readColor(st.fill);c.alpha*=Number(st.fillOpacity);if(c.alpha)fill={type:'solid',color:c};}
   const stroke=st.stroke!=='none'?{color:readColor(st.stroke),width:parseFloat(st.strokeWidth)*Math.sqrt(Math.abs(m.a*m.d-m.b*m.c)),dash:st.strokeDasharray!=='none'}:undefined;if(stroke)stroke.color.alpha*=Number(st.strokeOpacity);
   push({kind:'path',name:styleName(child),x:0,y:0,w:width,h:height,opacity:svgOpacity(child),fill,stroke,path:d});
  }

 }
 function text(node,style,opacity,name,clip){
  const value=node.textContent;if(!value?.trim())return;
  if((textCount+=value.length)>150000)throw Error('画板文字超过 15 万字，无法导出');
  const range=document.createRange(),requestedFont=style.fontFamily.split(',')[0].trim().replace(/^["']|["']$/g,''),fontSize=parseFloat(style.fontSize);
  const primaryFont=({'sans-serif':'Arial',serif:'Times New Roman',monospace:'Courier New','system-ui':/Mac/.test(navigator.platform)?'Helvetica Neue':'Segoe UI','-apple-system':'Helvetica Neue',BlinkMacSystemFont:'Helvetica Neue'})[requestedFont]??requestedFont;
  if(primaryFont!==requestedFont&&/system|BlinkMac/.test(requestedFont))warn(`${name}：系统字体映射为 ${primaryFont}，接收端需安装对应字体`);
  const clippedText=style.backgroundClip==='text'||style.webkitBackgroundClip==='text',textFill=clippedText?splitCss(style.backgroundImage).map(g=>gradient(g,box(node.parentElement).w,box(node.parentElement).h)).find(Boolean):undefined;
  if(textFill)warn(`${name}：渐变文字按可编辑文本行映射，请核对跨行渐变`);
  let line=null;
  const flush=()=>{if(line&&line.text.trim())push(line);line=null;};
  for(let i=0;i<value.length;){const char=String.fromCodePoint(value.codePointAt(i)),end=i+char.length;range.setStart(node,i);range.setEnd(node,end);const r=range.getBoundingClientRect(),b={x:r.x-origin.x,y:r.y-origin.y,w:r.width,h:r.height};i=end;
   if(!b.w&&/\s/.test(char))continue;
   if(!b.h||!visible(b)||b.x<clip.x-.5||b.y<clip.y-.5||b.x+b.w>clip.x+clip.w+.5||b.y+b.h>clip.y+clip.h+.5){flush();continue;}
   let str=char;if(style.textTransform==='uppercase')str=str.toLocaleUpperCase();if(style.textTransform==='lowercase')str=str.toLocaleLowerCase();
   const font=/[\u3000-\u9fff\uac00-\ud7af\uf900-\ufaff]/.test(char)&&!/PingFang|YaHei|SimSun|Songti|Noto.*(?:SC|CJK)|黑体|宋体/.test(primaryFont)?cjkFont(style):primaryFont;
   if(!line||line.font!==font||Math.abs(line.y-b.y)>1||Math.abs(line.h-b.h)>1||b.x>line.x+line.w+Math.max(3,fontSize/2)||(style.direction!=='rtl'&&b.x<line.x-1)){flush();line={kind:'text',name,x:b.x,y:b.y,w:Math.max(.1,b.w),h:b.h,opacity,text:str,font,fontSize,bold:parseInt(style.fontWeight)>=600,italic:style.fontStyle==='italic',underline:style.textDecorationLine.includes('underline'),strike:style.textDecorationLine.includes('line-through'),letterSpacing:parseFloat(style.letterSpacing)||0,rtl:style.direction==='rtl',fill:{type:'solid',color:readColor(style.color)}};}
   else{line.text+=str;const end=Math.max(line.x+line.w,b.x+b.w);line.x=Math.min(line.x,b.x);line.w=end-line.x;}
   if(line){if(textFill)line.fill=textFill;line.shadows=shadows(style.textShadow);}
  }flush();range.detach();
 }
 async function walk(el,opacity=1,clip={x:0,y:0,w:width,h:height},transformed=false){
  if(++count>30000)throw Error('画板 DOM 元素超过 30000 个');
  if(el.matches(excluded))return;
  const st=getComputedStyle(el),b=box(el),name=styleName(el);if(st.display==='none'||st.visibility==='hidden'||st.visibility==='collapse')return;
  if(!transformed&&st.transform&&st.transform!=='none'&&el!==document.body){
   const matrix=new DOMMatrix(st.transform),originValues=st.transformOrigin.split(' ').map(parseFloat);
   const sx=Math.hypot(matrix.a,matrix.b),sy=Math.hypot(matrix.c,matrix.d),orthogonal=Math.abs(matrix.a*matrix.c+matrix.b*matrix.d)<.001;
   if(matrix.is2D&&orthogonal&&sx>0&&sy>0&&matrix.a*matrix.d-matrix.b*matrix.c>0){
    const start=nodes.length,previous=el.style.getPropertyValue('transform'),priority=el.style.getPropertyPriority('transform');el.style.setProperty('transform','none','important');
    try{
     const untransformed=box(el),ox=untransformed.x+originValues[0],oy=untransformed.y+originValues[1],tx=matrix.e+ox-matrix.a*ox-matrix.c*oy,ty=matrix.f+oy-matrix.b*ox-matrix.d*oy;
     await walk(el,opacity,clip,true);
     for(const n of nodes.slice(start)){
      if(n.kind==='path'){
       n.path=svgpath(n.path).translate(n.x,n.y).matrix([matrix.a,matrix.b,matrix.c,matrix.d,tx,ty]).round(4).toString();n.x=n.y=0;n.w=width;n.h=height;
      }else{
       const cx=n.x+n.w/2,cy=n.y+n.h/2;n.w*=sx;n.h*=sy;n.x=matrix.a*cx+matrix.c*cy+tx-n.w/2;n.y=matrix.b*cx+matrix.d*cy+ty-n.h/2;n.rotation=(n.rotation??0)+Math.atan2(matrix.b,matrix.a)*180/Math.PI;
       if(n.radii)n.radii=n.radii.map(r=>r*Math.min(sx,sy));if(n.fontSize)n.fontSize*=sy;if(n.letterSpacing)n.letterSpacing*=sx;if(n.stroke)n.stroke.width*=Math.min(sx,sy);
      }
     }
    }finally{if(previous)el.style.setProperty('transform',previous,priority);else el.style.removeProperty('transform');}
    return;
   }
  }
  opacity*=Number(st.opacity);if(!opacity)return;
  if(el!==document.body&&(!visible(b)||b.w<=0||b.h<=0)&&st.display!=='contents'){if(st.overflow==='visible'&&st.display!=='none'){}else return;}
  const radii=['TopLeft','TopRight','BottomRight','BottomLeft'].map(k=>{const v=st[`border${k}Radius`];return Math.min(b.w/2,b.h/2,parseFloat(v)*(v?.includes('%')?Math.min(b.w,b.h)/100:1))||0;});
  const bg=st.backgroundImage,transform=st.transform;
  const clipped=st.clipPath&&st.clipPath!=='none'||st.maskImage&&st.maskImage!=='none';
  const rotated=transform!=='none'&&transform&&!/^matrix\(1, 0, 0, 1,/.test(transform);
  if(el!==document.body&&(clipped||st.filter!=='none'||st.backdropFilter&&st.backdropFilter!=='none'||st.mixBlendMode!=='normal'||rotated)){
   await bitmap(el,b,opacity,name,clipped?'复杂裁剪或蒙版':rotated?'旋转或非平移变换':'滤镜或混合效果');return;
  }
  if(el.localName==='svg'){await svg(el,opacity);return;}
  if(['canvas','video','iframe'].includes(el.localName)){await bitmap(el,b,opacity,name,'动态画面快照');return;}
  const ss=shadows(st.boxShadow);if(ss.some(s=>s.spread!==0))warn(`${name}：阴影扩展半径在 PowerPoint 中近似处理`);
  const layers=st.backgroundClip==='text'||st.webkitBackgroundClip==='text'?[]:fills(st,b);if(el===document.body&&!layers.length)layers.push({type:'solid',color:readColor(getComputedStyle(document.documentElement).backgroundColor==='rgba(0, 0, 0, 0)'?'white':getComputedStyle(document.documentElement).backgroundColor)});
  for(let i=0;i<layers.length;i++)push({kind:'shape',name,x:el===document.body?0:b.x,y:el===document.body?0:b.y,w:el===document.body?width:b.w,h:el===document.body?height:b.h,opacity,fill:layers[i],radii,shadows:i===0?ss:[]});
  const borderKeys=['Top','Right','Bottom','Left'],borders=borderKeys.map(k=>({width:parseFloat(st[`border${k}Width`])||0,color:readColor(st[`border${k}Color`]),dash:['dashed','dotted'].includes(st[`border${k}Style`])}));
  if(borders.some(t=>t.width&&t.color.alpha)){
   if(parseFloat(st.width)===0&&parseFloat(st.height)===0){
    const cx=borders[3].width,cy=borders[0].width,corners=[[0,0],[b.w,0],[b.w,b.h],[0,b.h]];
    borders.forEach((t,i)=>{if(t.width&&t.color.alpha){const a=corners[i],z=corners[(i+1)%4];push({kind:'path',name:name+' 三角形',...b,opacity,fill:{type:'solid',color:t.color},path:`M${a[0]} ${a[1]} L${z[0]} ${z[1]} L${cx} ${cy} Z`});}});
   }else if(borders.every(t=>JSON.stringify(t)===JSON.stringify(borders[0])))push({kind:'shape',name:name+' 边框',...b,opacity,radii,stroke:borders[0],shadows:layers.length?[]:ss});
   else borders.forEach((t,i)=>{if(!t.width||!t.color.alpha)return;const boxes=[{x:b.x,y:b.y,w:b.w,h:t.width},{x:b.x+b.w-t.width,y:b.y,w:t.width,h:b.h},{x:b.x,y:b.y+b.h-t.width,w:b.w,h:t.width},{x:b.x,y:b.y,w:t.width,h:b.h}];push({kind:'shape',name:name+' 边框',...boxes[i],opacity,fill:{type:'solid',color:t.color}});});
  }
  if(el.localName==='img'){
   if(el.naturalWidth&&el.naturalHeight){const canvas=document.createElement('canvas');canvas.width=Math.max(1,Math.ceil(b.w));canvas.height=Math.max(1,Math.ceil(b.h));if(canvas.width*canvas.height>16777216)throw Error('图片像素超过限制');const ctx=canvas.getContext('2d');const iw=el.naturalWidth,ih=el.naturalHeight,fit=st.objectFit;let dw=b.w,dh=b.h;if(fit==='contain'||fit==='cover'){const ratio=Math[fit==='contain'?'min':'max'](b.w/iw,b.h/ih);dw=iw*ratio;dh=ih*ratio;}else if(fit==='none'){dw=iw;dh=ih;}
    const position=st.objectPosition.split(' ').map(v=>parseFloat(v)/100),dx=(b.w-dw)*(position[0]??.5),dy=(b.h-dh)*(position[1]??.5);ctx.drawImage(el,dx,dy,dw,dh);push({kind:'image',name,...b,opacity,radii,image:canvas.toDataURL('image/png')});canvas.width=canvas.height=0;
   }return;
  }
  if(['input','textarea','select'].includes(el.localName)){
   const size=parseFloat(st.fontSize),left=parseFloat(st.paddingLeft)+parseFloat(st.borderLeftWidth),top=parseFloat(st.paddingTop)+parseFloat(st.borderTopWidth);
   let value=el.localName==='select'?el.selectedOptions?.[0]?.textContent:el.value;
   if(el.type==='password')value='•'.repeat(Math.min(100,value?.length??0));
   if(el.type==='checkbox'||el.type==='radio')value=el.checked?'✓':'';
   if(value)push({kind:'text',name:name+' 当前值',x:b.x+left,y:b.y+top,w:Math.max(1,b.w-left-parseFloat(st.paddingRight)),h:Math.max(size,b.h-top-parseFloat(st.paddingBottom)),opacity,text:String(value).slice(0,100000),font:st.fontFamily.split(',')[0].replace(/["']/g,''),fontSize:size,bold:parseInt(st.fontWeight)>=600,fill:{type:'solid',color:readColor(st.color)}});
   return;
  }
  if(bg&&bg!=='none'&&splitCss(bg).some(s=>!gradient(s,b.w,b.h))){warn(`${name}：背景图片或特殊渐变已转为图片`);const image=await raster(el,Math.ceil(b.w),Math.ceil(b.h),true);push({kind:'image',name:name+' 背景',...b,opacity:1,image});}
  if(st.overflowX!=='visible'||st.overflowY!=='visible'){const x=Math.max(clip.x,b.x),y=Math.max(clip.y,b.y);clip={x,y,w:Math.max(0,Math.min(clip.x+clip.w,b.x+b.w)-x),h:Math.max(0,Math.min(clip.y+clip.h,b.y+b.h)-y)};}
  if(el.localName==='li'&&st.listStyleType!=='none'&&st.listStylePosition==='outside'){
   const index=[...el.parentElement.children].filter(n=>n.localName==='li').indexOf(el)+Number(el.parentElement.getAttribute('start')||1),size=parseFloat(st.fontSize);
   const marker=st.listStyleType==='decimal'?`${index}.`:st.listStyleType==='circle'?'◦':st.listStyleType==='square'?'▪':'•';
   push({kind:'text',name:name+' 列表标记',x:b.x-size*1.5,y:b.y,w:size*1.25,h:size*1.2,opacity,text:marker,font:st.fontFamily.split(',')[0].replace(/["']/g,''),fontSize:size,fill:{type:'solid',color:readColor(st.color)}});
  }
  // Stable z ordering within each container. Positioned stacking contexts stay together.
  const children=[...el.childNodes].map((node,index)=>({node,index,z:node.nodeType===1?Number(getComputedStyle(node).zIndex)||0:0})).sort((a,b)=>a.z-b.z||a.index-b.index);
  for(const {node} of children){if(node.nodeType===3)text(node,st,opacity,name,clip);else if(node.nodeType===1)await walk(node,opacity,clip);}
 }
 try{await walk(document.body);base.warnings=[...warnings];return base;}
 finally{for(const clean of pseudoCleanup.reverse())clean();pseudoStyle.remove();}
}
