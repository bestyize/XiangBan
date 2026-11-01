/** Keep desktop game coordinates upright even in a tall/narrow browser window. */
export function fitGameViewport(width:number, height:number, orientation:string, desktop:boolean) {
  const availableWidth=Math.max(0,width), availableHeight=Math.max(0,height)
  if (!desktop) return {width:availableWidth,height:availableHeight}
  // Landscape games resize their own canvas; only tall desktop windows need
  // a fitted frame to avoid triggering their phone rotation behavior.
  if (orientation === 'landscape' && availableWidth >= availableHeight) {
    return {width:availableWidth,height:availableHeight}
  }
  const landscape=orientation === 'landscape', ratio=landscape ? 16/9 : 3/4
  const fittedWidth=Math.min(availableWidth,availableHeight*ratio,landscape ? 1280 : 480)
  return {width:Math.floor(fittedWidth),height:Math.floor(fittedWidth/ratio)}
}
