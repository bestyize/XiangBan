import {toCanvas} from 'html-to-image';

// Rendering stays in the same opaque sandbox as the design program.
let capturing=false;
const token=()=>new URLSearchParams(location.search).get('token');
const reply=(requestId,result)=>parent.postMessage({channel:'open-claude-design',token:token(),type:'capture-result',requestId,...result},'*');
addEventListener('message',async event=>{
 const data=event.data;
 if(event.source!==parent||data?.channel!=='open-claude-design'||data.type!=='capture'||data.token!==token())return;
 const {requestId,width,height,scale}=data;
 if(typeof requestId!=='string'||requestId.length>80)return;
 if(capturing){reply(requestId,{error:'画板正在导出'});return;}
 if(![width,height].every(n=>Number.isInteger(n)&&n>0&&n<=8192)||![1,2].includes(scale)||width*height*scale*scale>16777216){reply(requestId,{error:'导出尺寸超过 8192 边长或 1600 万像素限制'});return;}
 capturing=true;
 let timer;
 try{
  const ready=Promise.all([document.fonts?.ready,...[...document.images].map(img=>img.decode())]);
  await Promise.race([ready,new Promise((_,reject)=>{timer=setTimeout(()=>reject(Error('画板图片或字体加载超时')),15000);})]);
  clearTimeout(timer);
  await new Promise(resolve=>setTimeout(resolve,0));
  const canvas=await toCanvas(document.body,{
   width,height,canvasWidth:width,canvasHeight:height,pixelRatio:scale,skipAutoScale:true,
   filter:node=>!(node instanceof Element)||!node.closest('[data-ocd-editor],[data-ocd-pin-layer]'),
  });
  const png=canvas.toDataURL('image/png');canvas.width=0;canvas.height=0;
  if(!png.startsWith('data:image/png;base64,')||png.length>64*1024*1024)throw Error('画板图片过大，无法导出');
  reply(requestId,{png,width:width*scale,height:height*scale});
 }catch(error){reply(requestId,{error:String(error?.message??error).slice(0,300)});}
 finally{clearTimeout(timer);capturing=false;}
});
