// @vitest-environment happy-dom
import { afterEach, expect, it, vi } from 'vitest';
import { hitImage } from './images.ts';
afterEach(()=>{vi.unstubAllGlobals();vi.restoreAllMocks();});
it('uses transformed visible pixels instead of the transparent image rectangle for hit testing',()=>{
 class Raster { naturalWidth=2;naturalHeight=1;onload?:()=>void;set src(_value:string){this.onload?.();} }
 vi.stubGlobal('Image',Raster);
 vi.stubGlobal('DOMPoint',class {constructor(public x:number,public y:number){}matrixTransform(){return {x:this.x-10,y:this.y-20};}});
 vi.spyOn(HTMLCanvasElement.prototype,'getContext').mockReturnValue({drawImage:vi.fn(),getImageData:()=>({data:new Uint8ClampedArray([0,0,0,0,1,2,3,255])})} as unknown as CanvasRenderingContext2D);
 const attributes:Record<string,string>={x:'0',y:'0',width:'100',height:'50',href:'data:image/webp;base64,UklGRgAAAABXRUJQ'};
 const element={getAttribute:(name:string)=>attributes[name]??null,getScreenCTM:()=>({inverse:()=>({})})} as unknown as SVGImageElement;
 expect(hitImage(element,35,40)).toBe(false);
 expect(hitImage(element,85,40)).toBe(true);
 expect(hitImage(element,110,40)).toBe(false);
 attributes.opacity='0';expect(hitImage(element,85,40)).toBe(false);
});
