// @vitest-environment happy-dom
import { beforeEach,afterEach,it,expect,vi } from 'vitest';
import { invoke } from '../bridge.ts';
import { mountCapture } from './mount.ts';
vi.mock('../bridge.ts',()=>({invoke:vi.fn()}));
const ctx={clearRect:vi.fn(),setTransform:vi.fn(),save:vi.fn(),restore:vi.fn(),strokeRect:vi.fn(),beginPath:vi.fn(),moveTo:vi.fn(),lineTo:vi.fn(),stroke:vi.fn(),closePath:vi.fn(),fill:vi.fn(),fillText:vi.fn(),clip:vi.fn(),drawImage:vi.fn(),strokeStyle:''};
const output='data:image/jpeg;base64,/9j/';
const el=(id:string)=>document.getElementById(id)!;
const tick=()=>new Promise(resolve=>setTimeout(resolve,0));
function pointer(type:string,x:number,y:number,target:HTMLElement=el('capture')){target.dispatchEvent(new PointerEvent(type,{clientX:x,clientY:y,pointerId:1,button:0,bubbles:true}));}
async function crop(){pointer('pointerdown',100,100);pointer('pointermove',400,300);pointer('pointerup',400,300);await tick();}
beforeEach(()=>{
  document.body.innerHTML='<div id="app"></div>';vi.clearAllMocks();
  vi.mocked(invoke).mockImplementation(async(command)=>command==='capture_frame'?{image:output,width:1024,height:768}:undefined);
  vi.spyOn(HTMLCanvasElement.prototype,'getContext').mockReturnValue(ctx as unknown as CanvasRenderingContext2D);
  vi.spyOn(HTMLCanvasElement.prototype,'toDataURL').mockReturnValue(output);
  vi.spyOn(HTMLImageElement.prototype,'src','set').mockImplementation(function(this:HTMLImageElement,value:string){if(value)queueMicrotask(()=>this.dispatchEvent(new Event('load')));});
  vi.spyOn(HTMLImageElement.prototype,'naturalWidth','get').mockReturnValue(2048);
  vi.spyOn(HTMLImageElement.prototype,'naturalHeight','get').mockReturnValue(1536);
  Object.assign(HTMLElement.prototype,{setPointerCapture(){},hasPointerCapture(){return false;},releasePointerCapture(){}});
  vi.stubGlobal('requestAnimationFrame',(callback:FrameRequestCallback)=>{setTimeout(()=>callback(0),0);return 1;});
  vi.stubGlobal('cancelAnimationFrame',()=>{});
});
afterEach(()=>{window.dispatchEvent(new Event('beforeunload'));vi.restoreAllMocks();vi.unstubAllGlobals();document.body.replaceChildren();});
it('loads a real snapshot before showing the overlay, initially waiting for selection',async()=>{
  await mountCapture();
  expect(vi.mocked(invoke).mock.calls.map(c=>c[0])).toEqual(['capture_frame','capture_show']);
  expect(el('selection').hidden).toBe(true);expect(el('dock').hidden).toBe(true);expect(el('shadeFull').hidden).toBe(false);
});
it('selects, moves and resizes without changing the frozen background',async()=>{
  await mountCapture();await crop();
  expect(el('dock').hidden).toBe(false);expect(el('selection').style.width).toBe('300px');
  pointer('pointerdown',200,150,el('selection'));pointer('pointermove',240,180);pointer('pointerup',240,180);
  expect(el('selection').style.left).toBe('140px');expect(el('selection').style.top).toBe('130px');
  const handle=document.querySelector<HTMLElement>('[data-edge="se"]')!;
  pointer('pointerdown',440,330,handle);pointer('pointermove',500,400);pointer('pointerup',500,400);
  expect(el('selection').style.width).toBe('360px');expect(el('selection').style.height).toBe('270px');
  expect(vi.mocked(invoke).mock.calls.filter(c=>c[0]==='capture_frame')).toHaveLength(1);
});
it('renders annotations, supports undo and excludes UI from the exported JPEG',async()=>{
  await mountCapture();await crop();
  document.querySelector<HTMLButtonElement>('[data-tool="rectangle"]')!.click();
  pointer('pointerdown',130,130,el('selection'));pointer('pointermove',250,220);pointer('pointerup',250,220);await tick();
  expect(ctx.strokeRect).toHaveBeenCalled();expect((el('undo') as HTMLButtonElement).disabled).toBe(false);
  el('undo').click();await tick();expect((el('undo') as HTMLButtonElement).disabled).toBe(true);
  ctx.drawImage.mockClear();el('confirm').click();await tick();
  expect(invoke).toHaveBeenCalledWith('capture_finish',{image:output,error:null});
  expect(ctx.drawImage.mock.calls.map(c=>c[0])).toEqual([el('desktop'),el('marks')]);
});
it('uses rainbow pink by default and keeps the rainbow annotation colors selectable',async()=>{
  await mountCapture();await crop();
  const swatches=[...document.querySelectorAll<HTMLButtonElement>('[data-color]')];
  expect(swatches.map(item=>item.dataset.color)).toEqual(['#f26baa','#ffe47a','#55beeb','#ffffff','#37323b']);
  expect(swatches[0].getAttribute('aria-pressed')).toBe('true');
  document.querySelector<HTMLButtonElement>('[data-tool="rectangle"]')!.click();
  expect(el('palette').hidden).toBe(false);
  pointer('pointerdown',130,130,el('selection'));pointer('pointermove',250,220);pointer('pointerup',250,220);await tick();
  expect(ctx.strokeStyle).toBe('#f26baa');
  swatches[2].click();
  expect(swatches[2].getAttribute('aria-pressed')).toBe('true');
  expect(swatches[0].getAttribute('aria-pressed')).toBe('false');
  pointer('pointerdown',140,140,el('selection'));pointer('pointermove',260,240);pointer('pointerup',260,240);await tick();
  expect(ctx.strokeStyle).toBe('#55beeb');
});
it('cancels by Escape without sending an image',async()=>{
  await mountCapture();document.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape'}));await tick();
  expect(invoke).toHaveBeenCalledWith('capture_finish',{image:null,error:null});
});
it('clears selection and annotations on reselect',async()=>{
  await mountCapture();await crop();el('reselect').click();await tick();
  expect(el('selection').hidden).toBe(true);expect(el('dock').hidden).toBe(true);
  expect(vi.mocked(invoke).mock.calls.every(c=>c[0]!=='capture_finish')).toBe(true);
});
it('keeps IME Enter inside the text editor and allows cancelling only that text',async()=>{
  await mountCapture();await crop();document.querySelector<HTMLButtonElement>('[data-tool="text"]')!.click();
  pointer('pointerdown',140,140,el('selection'));
  const input=document.querySelector<HTMLTextAreaElement>('textarea')!;input.value='说明';
  input.dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',isComposing:true,bubbles:true}));expect(document.querySelector('textarea')).toBe(input);
  input.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true}));expect(document.querySelector('textarea')).toBeNull();
  expect(vi.mocked(invoke).mock.calls.every(c=>c[0]!=='capture_finish')).toBe(true);
});
it('allows retry after a native export rejection',async()=>{
  await mountCapture();await crop();vi.mocked(invoke).mockRejectedValueOnce('截图结果无效');
  el('confirm').click();await tick();expect(el('captureError').hidden).toBe(false);
  el('cancel').click();await tick();expect(invoke).toHaveBeenLastCalledWith('capture_finish',{image:null,error:null});
});
