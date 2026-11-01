import type {Board} from './model.ts';

export function exportOrder(boards:Board[]){return [...boards].sort((a,b)=>a.y-b.y||a.x-b.x);}
export function validatePng(png:string,board:Board,scale:1|2){
 if(typeof png!=='string'||png.length>64*1024*1024||!png.startsWith('data:image/png;base64,'))throw Error('导出图片无效或超过 48MiB');
 let bytes:Uint8Array;try{bytes=Uint8Array.from(atob(png.slice(22,66)),c=>c.charCodeAt(0));}catch{throw Error('PNG 编码无效');}
 if(bytes.length<24||[137,80,78,71,13,10,26,10].some((b,i)=>bytes[i]!==b)||String.fromCharCode(...bytes.slice(12,16))!=='IHDR')throw Error('不是有效的 PNG');
 const data=new DataView(bytes.buffer),width=data.getUint32(16),height=data.getUint32(20);
 if(width!==board.w*scale||height!==board.h*scale||width*height>16777216)throw Error('导出图片尺寸与画板不一致');
 return {width,height};
}
export async function pdfImage(png:string,board:Board){
 const {width,height}=validatePng(png,board,1),image=new Image();image.src=png;
 let timer:ReturnType<typeof setTimeout>|undefined;
 try{
  await Promise.race([image.decode(),new Promise((_,reject)=>{timer=setTimeout(()=>reject(Error('导出图片解码超时')),15000);})]);
  if(image.naturalWidth!==width||image.naturalHeight!==height)throw Error('图片尺寸不一致');
  const canvas=document.createElement('canvas');canvas.width=width;canvas.height=height;
  const context=canvas.getContext('2d');if(!context)throw Error('当前环境不能生成 PDF 图片');
  context.fillStyle='#ffffff';context.fillRect(0,0,width,height);context.drawImage(image,0,0);
  const jpeg=canvas.toDataURL('image/jpeg',.95);canvas.width=0;canvas.height=0;
  return {board,jpeg};
 }finally{clearTimeout(timer);image.src='';}
}
export async function makePdf(pages:{board:Board;jpeg:string}[]){
 if(!pages.length||pages.length>100)throw Error('请选择 1 到 100 个画板');
 const {jsPDF}=await import('jspdf');
 const size=(b:Board):[number,number]=>[b.w*.75,b.h*.75];
 const orientation=(b:Board)=>b.w>b.h?'landscape' as const:'portrait' as const;
 const doc=new jsPDF({unit:'pt',format:size(pages[0].board),orientation:orientation(pages[0].board),compress:true});
 pages.forEach(({board,jpeg},index)=>{if(index)doc.addPage(size(board),orientation(board));doc.addImage(jpeg,'JPEG',0,0,...size(board),undefined,'FAST');});
 const data=doc.output('datauristring').split(',')[1];
 if(data.length>64*1024*1024)throw Error('PDF 超过 48MiB，请减少画板数量');
 return 'data:application/pdf;base64,'+data;
}
