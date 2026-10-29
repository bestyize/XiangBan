export interface HitRegion { left:number;bottom:number;width:number;height:number;radius:number }
/** Keep regions anchored to the bottom while native fit changes the window height. */
export function hitRegion(rect:Pick<DOMRect,'left'|'bottom'|'width'|'height'>,viewportHeight:number,radius:number,padding=4):HitRegion|null {
  if(rect.width<=0||rect.height<=0)return null;
  return {left:rect.left-padding,bottom:viewportHeight-rect.bottom-padding,width:rect.width+padding*2,height:rect.height+padding*2,radius:radius+padding};
}
