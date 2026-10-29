export const clampScale=(value:number)=>Math.max(.1,Math.min(8,value));
export function zoomAt(scale:number,next:number,left:number,top:number,x:number,y:number){
 next=clampScale(next);const ratio=next/scale;
 return {scale:next,left:Math.max(0,(left+x)*ratio-x),top:Math.max(0,(top+y)*ratio-y)};
}
export function wheelPixels(delta:number,mode:number,page:number){return delta*(mode===1?16:mode===2?page:1);}
export interface CanvasView {selected?:string;page?:string;scale?:number;left?:number;top?:number;mode?:'select'|'play'|'hand';theme?:'light'|'dark'}
