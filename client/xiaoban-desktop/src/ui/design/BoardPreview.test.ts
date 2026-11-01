// @vitest-environment happy-dom
import {mount,flushPromises} from '@vue/test-utils';
import {describe,it,expect,vi,afterEach} from 'vitest';
import BoardPreview from './BoardPreview.vue';
afterEach(()=>vi.unstubAllGlobals());
describe('sandbox capture protocol',()=>{
 it('waits for readiness and only accepts the matching frame, token and request',async()=>{
  vi.stubGlobal('IntersectionObserver',undefined);
  const w=mount(BoardPreview,{attachTo:document.body,props:{src:'about:blank',title:'a',editing:true,token:'secret'}});await flushPromises();
  const frame=w.get('iframe').element as HTMLIFrameElement,post=vi.spyOn(frame.contentWindow!,'postMessage'),resolved=vi.fn();
  const result=(w.vm as any).capture({width:100,height:80,scale:2}).then(resolved);await flushPromises();
  expect(post.mock.calls.some(([data])=>(data as any).type==='capture')).toBe(false);
  const send=(data:any,source:any=frame.contentWindow)=>window.dispatchEvent(new MessageEvent('message',{source,data:{channel:'open-claude-design',token:'secret',...data}}));
  send({type:'ready'});const request=post.mock.calls.find(([data])=>(data as any).type==='capture')![0] as any;
  send({type:'capture-result',requestId:request.requestId,png:'spoof'},window);send({type:'capture-result',requestId:request.requestId,png:'spoof',token:'wrong'});send({type:'capture-result',requestId:'old',png:'spoof'});await flushPromises();expect(resolved).not.toHaveBeenCalled();
  send({type:'capture-result',requestId:request.requestId,png:'data:image/png;base64,AAAA'});await result;expect(resolved).toHaveBeenCalledWith('data:image/png;base64,AAAA');w.unmount();
 });
 it('rejects pending captures immediately when cancelled or unmounted',async()=>{
  vi.stubGlobal('IntersectionObserver',undefined);const w=mount(BoardPreview,{attachTo:document.body,props:{src:'about:blank',title:'a',editing:true,token:'s'}});await flushPromises();
  const pending=(w.vm as any).capture({width:100,height:80,scale:1});const rejected=expect(pending).rejects.toThrow('取消');w.unmount();await rejected;
 });
});
