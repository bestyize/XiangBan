// @vitest-environment happy-dom
import { mount, flushPromises, type VueWrapper } from '@vue/test-utils';
import { beforeEach, afterEach, describe, it, expect, vi } from 'vitest';
import CloudDocumentWindow from './CloudDocumentWindow.vue';
import { invoke } from '../../bridge.ts';
vi.mock('../../bridge.ts',()=>({invoke:vi.fn(),listen:vi.fn().mockResolvedValue(()=>{})}));
const snapshot={text:'<script>外部文档文字</script>',state:{coverage:'partial',note:'仅有摘要，不是全文',preview:'摘要',bytes:30,updatedMs:1}};
const link={id:'saved',title:'云端资料',platform:'baidu'};
let wrapper:VueWrapper | undefined;
beforeEach(()=>{
  window.history.replaceState(null,'','/?scene=configuration&kind=cloud&provider=saved');
  vi.mocked(invoke).mockReset().mockImplementation(async(command)=>command==='library_cloud_item'?{link,content:snapshot}:command==='get_settings'?{appearance:'light',accent:'green'}:null);
});
afterEach(()=>{wrapper?.unmount();wrapper=undefined;});
async function open(){wrapper=mount(CloudDocumentWindow,{global:{stubs:{WindowChrome:true,WindowResizeEdges:true}}});await flushPromises();return wrapper;}
describe('native cloud document preview',()=>{
  it('shows the cached body offline, escapes external markup, and prepares a scoped task',async()=>{
    const w=await open();expect(w.get('[data-testid="cloud-document-body"]').text()).toContain('<script>');expect(w.find('script').exists()).toBe(false);
    expect(w.text()).toContain('摘要／部分内容');expect(invoke).not.toHaveBeenCalledWith('library_cloud_read',expect.anything());
    await w.findAll('button').find(b=>b.text()==='带入提问')!.trigger('click');await flushPromises();
    expect(invoke).toHaveBeenCalledWith('library_cloud_open_task',{id:'saved',mode:'ask'});
    expect(invoke).not.toHaveBeenCalledWith('chat_send',expect.anything());
  });
  it('keeps the previous body visible when a refresh fails',async()=>{
    const w=await open();vi.mocked(invoke).mockRejectedValueOnce('云端读取超时');
    await w.findAll('button').find(b=>b.text()==='刷新内容')!.trigger('click');await flushPromises();
    expect(w.get('[role="alert"]').text()).toContain('云端读取超时');expect(w.get('[data-testid="cloud-document-body"]').text()).toBe(snapshot.text);
  });
  it('disables body questions when the service returns no readable text',async()=>{
    vi.mocked(invoke).mockImplementation(async(command)=>command==='library_cloud_item'?{link,content:{...snapshot,text:'',state:{...snapshot.state,coverage:'empty'}}}:command==='get_settings'?{appearance:'light'}:null);
    const w=await open();expect(w.findAll('button').find(b=>b.text()==='带入提问')!.attributes('disabled')).toBeDefined();
  });
});
