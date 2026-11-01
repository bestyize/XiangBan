// @vitest-environment happy-dom
import { mount, flushPromises, type VueWrapper } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { reactive } from 'vue';
import LibraryCloud from './LibraryCloud.vue';
import { emptyLibrary, type LibraryStore } from './library-store.ts';
import { invoke } from '../../bridge.ts';
vi.mock('../../bridge.ts',()=>({invoke:vi.fn(),listen:vi.fn().mockResolvedValue(()=>{})}));
vi.mock('./confirm.ts',()=>({useConfirm:()=>vi.fn()}));
const source={provider:'tencent',name:'腾讯文档',connectorId:'docs',status:'connected',connected:true,browse:true,search:true};
const entry={id:'file-1',name:'会议纪要',url:'https://docs.qq.com/doc/file-1',path:'',directory:false};
let wrapper:VueWrapper; let library:LibraryStore;
beforeEach(()=>{
  library=reactive({data:emptyLibrary(),busy:false,addLink:vi.fn(),remove:vi.fn(),renameLink:vi.fn()}) as unknown as LibraryStore;
  vi.mocked(invoke).mockReset().mockImplementation(async(command)=>{
    if(command==='library_cloud_sources')return[source];
    if(command==='library_cloud_browse')return{items:[entry],hasMore:false};
    if(command==='library_cloud_add')return{...emptyLibrary(),links:[{id:'saved',url:entry.url,title:entry.name,platform:'tencent',kind:'云端文件',code:null,folder:null,addedMs:1,remote:{provider:'tencent',id:entry.id,path:''}}]};
    return null;
  });
});
afterEach(()=>wrapper?.unmount());
async function open(){wrapper=mount(LibraryCloud,{props:{library,desktop:true,query:'',openUrl:vi.fn()}});await flushPromises();}
async function click(text:string){await wrapper.findAll('button').find(button=>button.text()===text)!.trigger('click');await flushPromises();}
describe('cloud library integration',()=>{
  it('browses an already connected account, saves a selected reference and opens its scoped native reader',async()=>{
    await open(); expect(wrapper.text()).toContain('已连接');
    await click('浏览文件'); expect(wrapper.text()).toContain('会议纪要');
    await click('读取并加入'); expect(invoke).toHaveBeenCalledWith('library_cloud_add',{platform:'tencent',entry,folder:null});
    expect(wrapper.text()).toContain('已加入');
    await wrapper.get('[aria-label="返回云文档"]').trigger('click');
    await wrapper.get('[aria-label="预览云文档与提问"]').trigger('click');
    expect(invoke).toHaveBeenCalledWith('configuration_window_open', {target:'cloud',provider:'saved',tab:null});
    expect(wrapper.emitted('ask')).toBeUndefined();
    expect(invoke).not.toHaveBeenCalledWith('chat_send',expect.anything());
  });
  it('reuses the existing native connection window and does not request credentials in the library',async()=>{
    await open(); await click('管理');
    expect(invoke).toHaveBeenCalledWith('configuration_window_open',expect.objectContaining({target:'store'}));
    expect(wrapper.find('input[type="password"]').exists()).toBe(false);
  });
  it('shows a retryable read error and does not add a fake result',async()=>{
    await open(); vi.mocked(invoke).mockRejectedValueOnce('连接已过期');
    await click('浏览文件'); expect(wrapper.text()).toContain('连接已过期');
    expect(wrapper.findAll('.cloud-file')).toHaveLength(0); await click('重试');
    expect(wrapper.text()).toContain('会议纪要');
  });
  it('waits for the visible read confirmation before dispatching a marked operation',async()=>{
    await open(); vi.mocked(invoke).mockResolvedValueOnce({items:[],hasMore:false,confirmationRequired:true});
    await click('浏览文件'); expect(wrapper.text()).toContain('本次只查询文件名与位置');
    expect(vi.mocked(invoke).mock.calls.some(([,args])=>args?.confirmed===true)).toBe(false);
    await click('确认读取');
    expect(invoke).toHaveBeenLastCalledWith('library_cloud_browse',expect.objectContaining({confirmed:true}));
    expect(wrapper.text()).toContain('会议纪要');
  });
});
