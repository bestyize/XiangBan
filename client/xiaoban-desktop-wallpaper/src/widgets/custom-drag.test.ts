// @vitest-environment happy-dom
import {describe,it,expect,vi} from 'vitest';
import {customWidgetDocument,installCustomDrag,isCustomDragMessage} from './custom-drag.ts';

function frame(html='<div id="card"><svg><path/></svg></div>') {
  const doc=document.implementation.createHTMLDocument();
  doc.body.innerHTML=html;
  const postMessage=vi.fn();
  // Execute exactly the standalone code injected into srcdoc, with an isolated document.
  new Function('window','document',`(${installCustomDrag.toString()})();`)({parent:{postMessage},innerWidth:280,innerHeight:240},doc);
  const pointer=(type:string,target:Element,x=70,y=60,id=1)=>target.dispatchEvent(new PointerEvent(type,{bubbles:true,cancelable:true,pointerId:id,button:0,clientX:x,clientY:y,screenX:100+x,screenY:100+y}));
  return {doc,postMessage,pointer,card:doc.querySelector('#card')!};
}

describe('custom widget iframe dragging',()=>{
  it('relays a background drag, captures the pointer and suppresses the resulting click',()=>{
    const {doc,postMessage,pointer}=frame();
    const path=doc.querySelector('path')!;
    const capture=vi.fn();Object.assign(path,{setPointerCapture:capture});
    const click=vi.fn();path.addEventListener('click',click);
    pointer('pointerdown',path);
    pointer('pointermove',path,72);
    expect(postMessage).toHaveBeenCalledTimes(1);
    pointer('pointermove',path,90);
    pointer('pointerup',path,95);
    expect(capture).toHaveBeenCalledWith(1);
    expect(postMessage.mock.calls.map(c=>c[0].phase)).toEqual(['start','move','end']);
    expect(postMessage.mock.calls[0][0]).toMatchObject({x:.25,y:.25});
    path.dispatchEvent(new MouseEvent('click',{bubbles:true,cancelable:true}));
    expect(click).not.toHaveBeenCalled();
  });
  it('preserves a tap on a clickable illustration',()=>{
    const {card,pointer,postMessage}=frame();const click=vi.fn();card.addEventListener('click',click);
    pointer('pointerdown',card);pointer('pointerup',card);
    card.dispatchEvent(new MouseEvent('click',{bubbles:true}));
    expect(click).toHaveBeenCalledOnce();
    expect(postMessage.mock.calls.map(c=>c[0].phase)).toEqual(['start','end']);
  });
  it('leaves native controls, nested button content, editing and custom gestures alone',()=>{
    const {doc,pointer,postMessage}=frame('<button><span>A</span></button><input><textarea></textarea><a href="#">link</a><div contenteditable><span>B</span></div><div data-no-drag><span>C</span></div><div role="button">D</div>');
    for(const element of doc.body.querySelectorAll('span,input,textarea,a,[role="button"]'))pointer('pointerdown',element);
    expect(postMessage).not.toHaveBeenCalled();
  });
  it('ignores other pointers and ends on cancellation or capture loss',()=>{
    for(const end of ['pointercancel','lostpointercapture']){
      const {card,pointer,postMessage}=frame();
      pointer('pointerdown',card);pointer('pointermove',card,90,60,2);
      expect(postMessage).toHaveBeenCalledTimes(1);
      pointer('pointermove',card,90);pointer(end,card,90);pointer('pointermove',card,100);
      expect(postMessage.mock.calls.map(c=>c[0].phase)).toEqual(['start','move','end']);
    }
  });
  it('injects before user scripts for full documents and fragments without changing the saved HTML',()=>{
    for(const html of ['<!DOCTYPE html><HTML><HEAD lang="en"><script>window.userScript=true</script></HEAD><body>Kitty</body></HTML>','<div>Kitty</div>']){
      const doc=new DOMParser().parseFromString(customWidgetDocument(html,true),'text/html');
      expect(doc.body.textContent).toContain('Kitty');
      expect(doc.head.querySelector('script')!.textContent).toContain('xiaoban-widget-drag');
      expect(doc.head.querySelector('style')!.textContent).toContain('background:transparent');
      expect(customWidgetDocument(html,false)).not.toContain('xiaoban-widget-drag');
    }
  });
  it('rejects invalid messages and grab coordinates',()=>{
    const message={type:'xiaoban-widget-drag',phase:'start',screenX:-100,screenY:20,x:.5,y:.5};
    expect(isCustomDragMessage(message)).toBe(true);
    for(const patch of [{type:'other'},{phase:'resize'},{x:-1},{y:2},{screenX:NaN},{screenY:'20'}])expect(isCustomDragMessage({...message,...patch})).toBe(false);
    expect(isCustomDragMessage(null)).toBe(false);
  });
});
