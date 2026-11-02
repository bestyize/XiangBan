// @vitest-environment happy-dom
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '../../bridge.ts';
import ChatAssets from './ChatAssets.vue';
import { conversationAssets, type LocalAsset } from './assets.ts';
import type { ReplyBlock } from './replies.ts';

vi.mock('../../bridge.ts',()=>({invoke:vi.fn(),listen:vi.fn(async()=>()=>{})}));
const image={id:'a'.repeat(64)+'.png',name:'生成的山野.png',mime:'image/png',width:1024,height:1024};
const file=(patch:Partial<LocalAsset>={}):LocalAsset=>({id:'file-1',name:'交付说明.md',path:'/work/交付说明.md',kind:'file',exists:true,imported:false,bytes:2048,updatedMs:1,toolCallId:'write',...patch});
const blocks:ReplyBlock[]=[{id:'reply-1',items:[],calls:[{kind:'tool',id:'write',name:'write_file',arguments:'{"path":"交付说明.md"}',status:'done',output:'ok'},{kind:'tool',id:'gen',name:'image_generate',arguments:'{}',status:'done',output:'ok',generatedImages:[image]}]}];
let wrapper:VueWrapper;
const ensureSession=vi.fn(async()=> 's1'),notify=vi.fn();
function open(props:Record<string,unknown>={}) {
  wrapper=mount(ChatAssets,{props:{session:'s1',designs:[],blocks,open:true,pinned:false,anchor:null,ensureSession,notify,...props},attachTo:document.body});
  return wrapper;
}
async function menuItem(name:string) {const b=wrapper.findAll('[role="menuitem"]').find(b=>b.text()===name);if(!b)throw Error(`missing ${name}`);await b.trigger('click');await flushPromises();}
beforeEach(()=>{vi.clearAllMocks();vi.mocked(invoke).mockImplementation(async command=>command==='chat_assets_list'?[file()]:command.includes('read')||command.includes('thumbnail')?'data:image/png;base64,YWJj':true);});
afterEach(()=>wrapper?.unmount());

describe('conversation asset integration',()=>{
  it('merges live image deliveries with local outputs, filters and opens native originals without regenerating',async()=>{
    open();await flushPromises();
    expect(wrapper.emitted('count')?.at(-1)).toEqual([2]);
    expect(wrapper.findAll('[data-testid="conversation-asset"]')).toHaveLength(2);
    await wrapper.get('[aria-label="搜索对话资产"]').setValue('山野');
    expect(wrapper.findAll('[data-testid="conversation-asset"]')).toHaveLength(1);
    await wrapper.get('[aria-label="打开生成的山野.png"]').trigger('click');await flushPromises();
    expect(invoke).toHaveBeenCalledWith('chat_generated_image_open',{id:image.id});
    expect(vi.mocked(invoke).mock.calls.some(([c])=>c==='image_generate')).toBe(false);
    await wrapper.get('[aria-label="搜索对话资产"]').setValue('');await wrapper.get('[data-tab="file"]').trigger('click');
    await wrapper.get('[aria-label="打开交付说明.md"]').trigger('click');await flushPromises();
    expect(invoke).toHaveBeenCalledWith('chat_asset_open',{sessionId:'s1',id:'file-1',relative:'',action:'open'});
    await wrapper.get('[aria-label="交付说明.md的更多操作"]').trigger('click');await menuItem('定位消息');
    expect(wrapper.emitted('locate')).toEqual([['reply-1']]);expect(wrapper.emitted('close')).toBeTruthy();
  });
  it('imports files/folders through native pickers and keeps cancelled imports quiet',async()=>{
    let imported=false;
    vi.mocked(invoke).mockImplementation(async (command)=>{if(command==='chat_assets_list')return imported?[file({imported:true,toolCallId:null})]:[];if(command==='chat_assets_import'){imported=true;return 1;}return true;});
    open({blocks:[]});await flushPromises();expect(wrapper.text()).toContain('这段对话还没有资产');
    await wrapper.get('[data-testid="chat-assets-import"]').trigger('click');await menuItem('导入文件');
    expect(invoke).toHaveBeenCalledWith('chat_assets_import',{sessionId:'s1',directory:false});expect(wrapper.text()).toContain('交付说明.md');expect(notify).toHaveBeenCalledWith('已导入 1 份资产','success');
    await wrapper.get('[data-testid="chat-assets-import"]').trigger('click');await menuItem('导入文件夹');expect(invoke).toHaveBeenCalledWith('chat_assets_import',{sessionId:'s1',directory:true});
    notify.mockClear();vi.mocked(invoke).mockImplementation(async c=>c==='chat_assets_import'?0:c==='chat_assets_list'?[file()]:true);
    await wrapper.get('[data-testid="chat-assets-import"]').trigger('click');await menuItem('导入文件');expect(notify).not.toHaveBeenCalled();
  });
  it('browses descendants using the original root ID, preserving the top-level count',async()=>{
    const folder=file({id:'root',kind:'folder',name:'交付目录',path:'/work/交付目录'});
    vi.mocked(invoke).mockImplementation(async c=>c==='chat_assets_list'?[folder]:c==='chat_asset_children'?{entries:[file({name:'报告.pdf',path:'/work/交付目录/报告.pdf'})],truncated:false}:true);
    open({blocks:[]});await flushPromises();await wrapper.get('[aria-label="打开交付目录"]').trigger('click');await flushPromises();
    expect(invoke).toHaveBeenCalledWith('chat_asset_children',{sessionId:'s1',id:'root',relative:''});
    await wrapper.get('[aria-label="打开报告.pdf"]').trigger('click');await flushPromises();expect(invoke).toHaveBeenCalledWith('chat_asset_open',{sessionId:'s1',id:'root',relative:'报告.pdf',action:'open'});
    expect(wrapper.emitted('count')?.at(-1)).toEqual([1]);await wrapper.get('[aria-label="返回上一级"]').trigger('click');expect(wrapper.text()).toContain('交付目录');
  });
  it('ignores old session responses and late import results after switching conversations',async()=>{
    let finishList!:(rows:LocalAsset[])=>void,finishImport!:(count:number)=>void;
    vi.mocked(invoke).mockImplementation(async (c,args)=>c==='chat_assets_list'?(args?.sessionId==='s1'?new Promise<LocalAsset[]>(resolve=>finishList=resolve):[]):c==='chat_assets_import'?new Promise<number>(resolve=>finishImport=resolve):true);
    open({blocks:[]});await wrapper.get('[data-testid="chat-assets-import"]').trigger('click');await menuItem('导入文件');
    await wrapper.setProps({session:'s2'});await flushPromises();finishList([file()]);finishImport(1);await flushPromises();
    expect(wrapper.text()).not.toContain('交付说明.md');expect(wrapper.emitted('count')?.at(-1)).toEqual([0]);
    expect(vi.mocked(invoke).mock.calls.filter(([c,a])=>c==='chat_assets_list'&&a?.sessionId==='s1')).toHaveLength(1);
  });
  it('keeps failed file actions actionable and disables saving missing originals',async()=>{
    vi.mocked(invoke).mockImplementation(async c=>{if(c==='chat_assets_list')return [file({exists:false})];if(c==='chat_asset_open')throw Error('原文件已移动或删除');return true;});
    open();await flushPromises();await wrapper.get('[aria-label="打开交付说明.md"]').trigger('click');await flushPromises();expect(wrapper.get('[role="alert"]').text()).toContain('原文件已移动或删除');
    await wrapper.get('[aria-label="交付说明.md的更多操作"]').trigger('click');expect(wrapper.findAll('[role="menuitem"]').find(b=>b.text()==='另存为')?.attributes('disabled')).toBeDefined();
    await menuItem('定位消息');expect(wrapper.emitted('locate')?.at(-1)).toEqual(['reply-1']);
  });
  it('imports editable design sources through the existing design importer',async()=>{
    vi.mocked(invoke).mockImplementation(async c=>c==='chat_assets_list'?[]:c==='design_import'?{project:{id:'design-imported'}}:true);
    open();await flushPromises();await wrapper.get('[data-testid="chat-assets-import"]').trigger('click');await menuItem('导入设计源文件');
    expect(invoke).toHaveBeenCalledWith('design_import',{session:'s1'});expect(wrapper.emitted('design')).toEqual([['design-imported']]);
  });
});

it('deduplicates reused native images and associates the most recent source message',()=>{
  const result=conversationAssets([],[],[...blocks,{...blocks[0],id:'reply-2'}]);
  expect(result).toHaveLength(1);expect(result[0].blockId).toBe('reply-2');
});

it('routes supported local files to the opted-in workspace and keeps Office files external',async()=>{
 vi.mocked(invoke).mockImplementation(async c=>c==='chat_assets_list'?[file({name:'demo.html',path:'/work/demo.html'}),file({id:'ppt',name:'deck.pptx',path:'/work/deck.pptx'})]:true);
 open({previewable:true,blocks:[]});await flushPromises();await wrapper.get('[aria-label="打开demo.html"]').trigger('click');await flushPromises();expect(wrapper.emitted('preview')?.[0]?.[0]).toMatchObject({id:'file-1',name:'demo.html'});expect(vi.mocked(invoke).mock.calls.some(([c])=>c==='chat_asset_open')).toBe(false);
 await wrapper.get('[aria-label="打开deck.pptx"]').trigger('click');await flushPromises();expect(invoke).toHaveBeenCalledWith('chat_asset_open',{sessionId:'s1',id:'ppt',relative:'',action:'open'});
});
