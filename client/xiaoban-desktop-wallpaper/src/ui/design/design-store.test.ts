// @vitest-environment happy-dom
import {beforeEach,describe,expect,it,vi} from 'vitest';
import {createDesignStore} from './design-store.ts';
import type {Project,Summary} from './model.ts';
const mocks=vi.hoisted(()=>({invoke:vi.fn(),listen:vi.fn()}));
vi.mock('../../bridge.ts',()=>mocks);
const fixture=():Project=>({id:'design-012345678901234567890123',title:'作品',sessionId:'s',revision:1,updatedMs:1,canvas:{v:3,boards:{'Main.dc.html':{w:390,h:844}},order:['Main.dc.html']},files:{'Main.dc.html':'<x-dc><h1>源文件</h1></x-dc>'}});
beforeEach(()=>{vi.resetAllMocks();mocks.listen.mockResolvedValue(()=>{});});
describe('shared versioned design store',()=>{
 it('backs up edits before CAS, retains a failed draft, and retries without losing manual changes',async()=>{
  const p=fixture();let fail=true;mocks.invoke.mockImplementation(async(command,args)=>{if(command==='design_read')return {project:p,origin:'preview:'};if(command==='design_save'){if(fail)throw Error('磁盘暂时不可写');return {project:{...args.project,revision:2},origin:'preview:'};}});
  const ds=createDesignStore(vi.fn());await ds.open(p.id);expect(await ds.commit(d=>{d.files['Main.dc.html']='手改源文件';})).toBe(false);expect(ds.dirty).toBe(true);expect(ds.project?.files['Main.dc.html']).toBe('手改源文件');expect(ds.conflict).toBe(false);
  const calls=mocks.invoke.mock.calls.map(c=>c[0]);expect(calls.indexOf('design_draft_save')).toBeLessThan(calls.indexOf('design_save'));
  fail=false;expect(await ds.save()).toBe(true);expect(ds.project?.revision).toBe(2);expect(ds.dirty).toBe(false);expect(mocks.invoke).toHaveBeenCalledWith('design_save',expect.objectContaining({expectedRevision:1}));
 });
 it('recovers old drafts, prevents overwriting newer AI work, and can explicitly discard the recovery file',async()=>{
  const p=fixture(),draft={...p,files:{'Main.dc.html':'未保存手改'}};let recovered=true;mocks.invoke.mockImplementation(async(command)=>{if(command==='design_read')return {project:{...p,revision:3},draft:recovered?draft:undefined,origin:'preview:'};if(command==='design_draft_clear')recovered=false;});
  const ds=createDesignStore(vi.fn());await ds.open(p.id);expect(ds.dirty&&ds.conflict).toBe(true);expect(await ds.commit(d=>{d.title='新标题';})).toBe(false);await ds.export('zip');expect(mocks.invoke).toHaveBeenCalledWith('design_export',{id:p.id,format:'zip',project:draft});await ds.discard();expect(ds.dirty).toBe(false);expect(ds.project?.revision).toBe(3);
 });
 it('does not replace unsaved UI work when a remote revision event arrives',async()=>{
  let changed:(s:Summary)=>void=()=>{};mocks.listen.mockImplementation(async(_,cb)=>{changed=cb;return ()=>{};});mocks.invoke.mockImplementation(async(command)=>{if(command==='design_read')return {project:fixture(),origin:'preview:'};if(command==='design_save')throw Error('保存失败');});
  const ds=createDesignStore(vi.fn());await ds.open(fixture().id);await ds.init();await ds.commit(p=>{p.title='用户修改';});changed({...fixture(),artboards:1,revision:2});expect(ds.project?.title).toBe('用户修改');expect(ds.conflict).toBe(true);expect(ds.error).toContain('新版本');ds.dispose();
 });
 it('ignores a late project read after the user switches to another design',async()=>{
  let resolve:(v:unknown)=>void=()=>{};mocks.invoke.mockImplementation((_,args)=>args.id==='slow'?new Promise(r=>{resolve=r;}):Promise.resolve({project:{...fixture(),id:'next'},origin:'preview:'}));
  const ds=createDesignStore(vi.fn()),old=ds.open('slow');await ds.open('next');resolve({project:fixture(),origin:'preview:'});await old;expect(ds.project?.id).toBe('next');
 });
});

describe('failed project navigation',()=>{
 it('clears the previous project before reading another id so edits cannot target the wrong source',async()=>{
  mocks.invoke.mockImplementation(async(_,args)=>{if(args.id==='missing')throw Error('不存在');return {project:fixture(),origin:'preview:'};});const ds=createDesignStore(vi.fn());await ds.open(fixture().id);await expect(ds.open('missing')).rejects.toThrow('不存在');expect(ds.project).toBeNull();expect(await ds.commit(p=>{p.title='错误的作品';})).toBe(false);
 });
});

describe('updates arriving while a save is completing',()=>{
 it('refreshes a newer AI revision after acknowledging its own save',async()=>{
  let changed:(s:Summary)=>void=()=>{},finish:(p:unknown)=>void=()=>{};let saved=fixture();mocks.listen.mockImplementation(async(_,cb)=>{changed=cb;return ()=>{};});
  mocks.invoke.mockImplementation(async(command)=>{if(command==='design_read')return {project:saved,origin:'preview:'};if(command==='design_save')return await new Promise(resolve=>{finish=resolve;});});
  const remote=vi.fn(),ds=createDesignStore(vi.fn());await ds.open(saved.id);await ds.init(remote);const saving=ds.commit(p=>{p.title='手改';});await Promise.resolve();await Promise.resolve();
  changed({...fixture(),revision:2,artboards:1});changed({...fixture(),revision:3,artboards:1});saved={...fixture(),revision:3,title:'最新 AI 版本'};finish({project:{...fixture(),revision:2,title:'手改'},origin:'preview:'});await saving;await Promise.resolve();await Promise.resolve();expect(ds.project?.revision).toBe(3);expect(ds.project?.title).toBe('最新 AI 版本');expect(ds.dirty).toBe(false);expect(remote).toHaveBeenCalledWith(expect.objectContaining({revision:3}));ds.dispose();
 });
});
