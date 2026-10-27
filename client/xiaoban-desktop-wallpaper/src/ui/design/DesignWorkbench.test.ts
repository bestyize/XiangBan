// @vitest-environment happy-dom
import {flushPromises,mount,type VueWrapper} from '@vue/test-utils';
import {beforeEach,afterEach,describe,expect,it,vi} from 'vitest';
import DesignWorkbench from './DesignWorkbench.vue';
import BoardPreview from './BoardPreview.vue';
import type {Project} from './model.ts';
const mocks=vi.hoisted(()=>({invoke:vi.fn(),listen:vi.fn()}));
vi.mock('../../bridge.ts',()=>mocks);
let wrapper:VueWrapper|undefined;let notify=vi.fn();
const fixture=():Project=>({id:'design-012345678901234567890123',title:'紧凑设计',sessionId:'s1',revision:1,updatedMs:1,canvas:{v:3,boards:{'Main.dc.html':{x:0,y:0,w:390,h:844,title:'首页'},'More.dc.html':{x:450,y:0,w:390,h:844,title:'更多'}},order:['Main.dc.html','More.dc.html'],notes:{}},files:{'Main.dc.html':'<x-dc><h1>源标题</h1></x-dc>','More.dc.html':'<x-dc><p>另一屏</p></x-dc>'}});
async function open(){notify=vi.fn();wrapper=mount(DesignWorkbench,{props:{projectId:fixture().id,notify}});await flushPromises();return wrapper;}
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
  const w=await open();w.findComponent(BoardPreview).vm.$emit('message',{type:'select',node:'n0',tag:'H1',text:'源标题'});await flushPromises();await click('添加批注');await w.get('.dialog textarea').setValue('把首页按钮缩小');await w.get('form.dialog').trigger('submit');await flushPromises();
  const save=mocks.invoke.mock.calls.find(([cmd])=>cmd==='design_save')![1];expect(save.project.canvas.xiaoban.comments[0]).toMatchObject({file:'Main.dc.html',text:'把首页按钮缩小',revision:1});expect(save.project.files['Main.dc.html']).toContain('data-ocd-node="n0"');const request=w.emitted('request')![0];expect(request[0]).toContain('先读取最新版');expect(request[1]).toBe('s1');
 });
 it('ignores preview messages from a different frame even with a matching token',async()=>{
  const w=mount(BoardPreview,{props:{src:'about:blank',title:'预览',editing:true,token:'secret'}});wrapper=w;await flushPromises();
  window.dispatchEvent(new MessageEvent('message',{source:window,data:{channel:'open-claude-design',token:'secret',type:'select',node:'n0'}}));expect(w.emitted('message')).toBeUndefined();
  const frame=w.get('iframe').element as HTMLIFrameElement;window.dispatchEvent(new MessageEvent('message',{source:frame.contentWindow,data:{channel:'open-claude-design',token:'wrong',type:'select'}}));expect(w.emitted('message')).toBeUndefined();
 });
 it('refuses to save an old source editor buffer after the AI updates that project',async()=>{
  const w=await open();await click('画板源代码');await w.get('.source-dialog textarea').setValue('旧源码草稿');
  mocks.invoke.mockImplementation(async(command)=>command==='design_read'?{project:{...fixture(),revision:2},origin:'design-preview://localhost'}:undefined);
  const changed=mocks.listen.mock.calls.find(([event])=>event==='design-changed')![1];changed({...fixture(),artboards:2,revision:2});await flushPromises();
  await w.get('.source-dialog .primary').trigger('click');await flushPromises();expect(mocks.invoke.mock.calls.some(([cmd])=>cmd==='design_save')).toBe(false);expect(notify).toHaveBeenCalledWith(expect.stringContaining('新版本'),'error');expect((w.get('.source-dialog textarea').element as HTMLTextAreaElement).value).toBe('旧源码草稿');
 });

 it('does not rebuild an obsolete undo entry when a newer AI revision arrives during saving',async()=>{
  const w=await open();let finish:(p:unknown)=>void=()=>{},draft:Project|undefined;
  mocks.invoke.mockImplementation(async(command,args)=>{if(command==='design_save'){draft=args.project;return await new Promise(resolve=>{finish=resolve;});}if(command==='design_read')return {project:{...draft!,revision:3,title:'AI 最新版本'},origin:'design-preview://localhost'};});
  await click('新增画板');const changed=mocks.listen.mock.calls.find(([event])=>event==='design-changed')![1];changed({...fixture(),revision:3,artboards:3});finish({project:{...draft!,revision:2},origin:'design-preview://localhost'});await flushPromises();
  expect(w.text()).toContain('AI 最新版本');expect((w.get('[title="撤销"]').element as HTMLButtonElement).disabled).toBe(true);
 });

 it('supports batch sending pending comments and manages design system tokens',async()=>{
  const withComments:Project={
    ...fixture(),
    canvas:{
      ...fixture().canvas,
      designSystems:[{name:'Brand',tokens:{primary:'#d94e60'}}],
      xiaoban:{
        comments:[
          {id:'c1',file:'Main.dc.html',node:'n0',text:'第一条批注',revision:1,status:'saved'},
          {id:'c2',file:'More.dc.html',node:'n1',text:'第二条批注',revision:1,status:'saved'},
        ]
      }
    }
  };
  mocks.invoke.mockImplementation(async(command,args)=>{
    if(command==='design_read')return {project:withComments,origin:'design-preview://localhost'};
    if(command==='design_save')return {project:{...args.project,revision:args.expectedRevision+1},origin:'design-preview://localhost'};
    if(command==='design_export')return 'exported.md';
  });
  const w=await open();
  await click('属性面板');
  expect(w.text()).toContain('全部发给小伴 (2)');
  await w.get('.primary-soft').trigger('click');
  await flushPromises();
  const req=w.emitted('request')![0];
  expect(req[0]).toContain('批量处理设计稿');
  expect(req[0]).toContain('1. [c1]');
  expect(req[0]).toContain('2. [c2]');

  const dsTab=w.findAll('.prop-tabs button').find(b=>b.text()==='设计系统')!;
  await dsTab.trigger('click');
  await flushPromises();
  expect(w.text()).toContain('--ds-primary');

  const exportBtn=w.findAll('.export div button').find(b=>b.text().includes('Markdown'))!;
  await exportBtn.trigger('click');
  await flushPromises();
  expect(mocks.invoke).toHaveBeenCalledWith('design_export',expect.objectContaining({format:'md'}));
 });

 it('opens incomplete drafts as placeholders without requesting missing preview files',async()=>{
  const draft=fixture();draft.status='generating';delete draft.files['More.dc.html'];
  mocks.invoke.mockResolvedValue({project:draft,origin:'design-preview://localhost'});
  const w=await open();expect(w.text()).toContain('生成中');expect(w.findAll('iframe')).toHaveLength(1);expect(w.text()).toContain('画板尚未生成');
 });

 it('saves canvas view before detaching, and offers docking in an independent window',async()=>{
  const w=await open();await click('独立窗口');
  expect(mocks.invoke).toHaveBeenCalledWith('design_view_save',expect.objectContaining({id:fixture().id,view:expect.objectContaining({scale:.4,selected:'Main.dc.html'})}));
  expect(w.emitted('detach')).toHaveLength(1);await w.setProps({detached:true});await click('回到分屏');expect(w.emitted('dock')).toHaveLength(1);
 });
 it('applies zoom shortcuts without intercepting source text entry',async()=>{
  const w=await open();const root=w.get('[data-testid="design-workbench"]');await root.trigger('keydown',{key:'+'});expect(w.get('.zoom').text()).toBe('48%');
  await root.trigger('keydown',{key:'1'});expect(w.get('.zoom').text()).toBe('100%');
  await click('画板源代码');await w.get('.source-dialog textarea').trigger('keydown',{key:'-'});expect(w.get('.zoom').text()).toBe('100%');
 });

 it('copies a project without changing the original and saves a title as a versioned edit',async()=>{
  const w=await open();const copied={...fixture(),id:'design-000000000000000000000000'};
  mocks.invoke.mockImplementation(async(cmd,args)=>cmd==='design_copy'?{project:copied}:cmd==='design_save'?{project:{...args.project,revision:args.expectedRevision+1},origin:'preview:'}:undefined);
  await click('创建副本');expect(w.emitted('copied')?.[0]).toEqual([copied.id]);expect(w.text()).toContain('紧凑设计');
  await click('重命名设计稿');await w.get('[aria-label="设计稿名称"]').setValue('新名字');await w.get('form.dialog').trigger('submit');await flushPromises();
  expect(mocks.invoke).toHaveBeenCalledWith('design_save',expect.objectContaining({project:expect.objectContaining({id:fixture().id,title:'新名字'}),expectedRevision:1}));
 });

});
