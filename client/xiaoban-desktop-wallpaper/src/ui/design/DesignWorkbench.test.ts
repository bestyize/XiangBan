// @vitest-environment happy-dom
import {flushPromises,mount,type VueWrapper} from '@vue/test-utils';
import {beforeEach,afterEach,describe,expect,it,vi} from 'vitest';
import DesignWorkbench from './DesignWorkbench.vue';
import BoardPreview from './BoardPreview.vue';
import type {Project} from './model.ts';
const mocks=vi.hoisted(()=>({invoke:vi.fn(),listen:vi.fn()}));
vi.mock('../../bridge.ts',()=>mocks);
let wrapper:VueWrapper|undefined;
const fixture=():Project=>({id:'design-012345678901234567890123',title:'紧凑设计',sessionId:'s1',revision:1,updatedMs:1,canvas:{v:3,boards:{'Main.dc.html':{x:0,y:0,w:390,h:844,title:'首页'},'More.dc.html':{x:450,y:0,w:390,h:844,title:'更多'}},order:['Main.dc.html','More.dc.html'],notes:{}},files:{'Main.dc.html':'<x-dc><h1>源标题</h1></x-dc>','More.dc.html':'<x-dc><p>另一屏</p></x-dc>'}});
async function open(){wrapper=mount(DesignWorkbench,{props:{projectId:fixture().id,notify:vi.fn()}});await flushPromises();return wrapper;}
async function click(title:string){await wrapper!.get(`[title="${title}"]`).trigger('click');await flushPromises();}
beforeEach(()=>{vi.resetAllMocks();vi.stubGlobal('IntersectionObserver',undefined);mocks.listen.mockResolvedValue(()=>{});mocks.invoke.mockImplementation(async(command,args)=>{if(command==='design_read')return {project:fixture(),origin:'design-preview://localhost'};if(command==='design_save')return {project:{...args.project,revision:args.expectedRevision+1},origin:'design-preview://localhost'};});});
afterEach(()=>{wrapper?.unmount();wrapper=undefined;vi.unstubAllGlobals();});
describe('compact design workbench',()=>{
 it('shows multiple sandboxed boards, starts with properties closed and saves additions with undo/redo',async()=>{
  const w=await open();expect(w.find('.properties').exists()).toBe(false);expect(w.findAll('.artboard')).toHaveLength(2);expect(w.findAll('iframe').every(f=>f.attributes('sandbox')==='allow-scripts')).toBe(true);
  await click('新增画板');expect(w.findAll('.artboard')).toHaveLength(3);expect(mocks.invoke).toHaveBeenCalledWith('design_save',expect.objectContaining({expectedRevision:1}));
  await click('撤销');expect(w.findAll('.artboard')).toHaveLength(2);await click('重做');expect(w.findAll('.artboard')).toHaveLength(3);
 });
 it('patches the selected source element and keeps other boards intact',async()=>{
  const w=await open();w.findComponent(BoardPreview).vm.$emit('message',{type:'select',node:'n0',tag:'H1',text:'源标题'});await flushPromises();
  const input=w.findAll('.properties label').find(l=>l.text()==='文字')!.get('input');await input.setValue('手改标题');await input.trigger('change');await flushPromises();
  const save=mocks.invoke.mock.calls.find(([cmd])=>cmd==='design_save')![1];expect(save.project.files['Main.dc.html']).toContain('手改标题');expect(save.project.files['More.dc.html']).toBe(fixture().files['More.dc.html']);expect(save.expectedRevision).toBe(1);
 });
 it('saves a contextual comment before asking the original chat to edit it',async()=>{
  const w=await open();await click('添加批注');await w.get('.dialog textarea').setValue('把首页按钮缩小');await w.get('form.dialog').trigger('submit');await flushPromises();
  const save=mocks.invoke.mock.calls.find(([cmd])=>cmd==='design_save')![1];expect(save.project.canvas.xiaoban.comments[0]).toMatchObject({file:'Main.dc.html',text:'把首页按钮缩小',revision:1});const request=w.emitted('request')![0];expect(request[0]).toContain('先读取最新版');expect(request[1]).toBe('s1');
 });
 it('ignores preview messages from a different frame even with a matching token',async()=>{
  const w=mount(BoardPreview,{props:{src:'about:blank',title:'预览',editing:true,token:'secret'}});wrapper=w;await flushPromises();
  window.dispatchEvent(new MessageEvent('message',{source:window,data:{channel:'open-claude-design',token:'secret',type:'select',node:'n0'}}));expect(w.emitted('message')).toBeUndefined();
  const frame=w.get('iframe').element as HTMLIFrameElement;window.dispatchEvent(new MessageEvent('message',{source:frame.contentWindow,data:{channel:'open-claude-design',token:'wrong',type:'select'}}));expect(w.emitted('message')).toBeUndefined();
 });
});
