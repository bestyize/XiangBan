import type {Project} from './model.ts';
export interface CanvasNote {id:string;x:number;y:number;w:number;maxH:number;text:string;kind:string;page?:string;[key:string]:unknown}
const number=(value:unknown,fallback:number,min:number,max:number)=>typeof value==='number'&&Number.isFinite(value)?Math.min(max,Math.max(min,value)):fallback;
export function notesOf(canvas:Project['canvas']):CanvasNote[]{
 if(!canvas.notes||Array.isArray(canvas.notes)||typeof canvas.notes!=='object')return [];
 return Object.entries(canvas.notes).filter(([,n])=>n&&typeof n==='object').map(([id,raw])=>{
  const n=raw as Record<string,unknown>,title=String(n.kind??'').startsWith('title');
  return {...n,id,x:number(n.x,0,-1000000,1000000),y:number(n.y,0,-1000000,1000000),w:number(title?n.maxW??n.w:n.w,320,120,16000),maxH:number(n.maxH,title?100:520,32,16000),kind:typeof n.kind==='string'?n.kind:'note',text:typeof n.text==='string'?n.text:'',page:typeof n.page==='string'?n.page:undefined};
 });
}
export function patchNote(p:Project,id:string,patch:Partial<CanvasNote>){
 const current=p.canvas.notes?.[id];if(!current)throw Error('便签已经删除');
 const changes={...patch};delete changes.id;
 if(current.kind?.startsWith('title')&&changes.w!==undefined){changes.maxW=changes.w;delete changes.w;}
 Object.assign(current,changes);
}
export function addNote(p:Project,x:number,y:number,page?:string,from?:CanvasNote):string{
 p.canvas.notes??={};if(Object.keys(p.canvas.notes).length>=300)throw Error('一份设计稿最多 300 个便签');
 const id='note-'+crypto.randomUUID();
 const note:Record<string,unknown>=from?{...from,x:from.x+24,y:from.y+24}:{x,y,w:320,maxH:520,text:'双击编辑便签',fill:'blue'};
 delete note.id;if(page)note.page=page;p.canvas.notes[id]=note;return id;
}
