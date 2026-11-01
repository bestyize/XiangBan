// @vitest-environment happy-dom
import {flushPromises,mount} from '@vue/test-utils';
import {beforeEach,describe,expect,it,vi} from 'vitest';
import DesignMcpSettings from './DesignMcpSettings.vue';
import {invoke} from '../../../bridge.ts';
vi.mock('../../../bridge.ts',()=>({desktop:true,invoke:vi.fn()}));
const ipc=vi.mocked(invoke);
const initial=()=>({running:false,enabled:false,url:'http://127.0.0.1:58140/mcp',error:null,clients:[]});
describe('design collaboration',()=>{
 beforeEach(()=>{ipc.mockReset();ipc.mockResolvedValue(initial());});
 it('does not enable or register clients on opening the settings',async()=>{const w=mount(DesignMcpSettings);await flushPromises();expect(ipc).toHaveBeenCalledExactlyOnceWith('design_mcp_status');expect(w.text()).toContain('本机服务未启动');w.unmount();});
 it('registers the selected client and distinguishes configuration from a real tool call',async()=>{ipc.mockImplementation(async(command)=>command==='design_mcp_configure'?{...initial(),running:true,clients:[{id:'codex',configured:true,lastCall:null}]}:initial());const w=mount(DesignMcpSettings);await flushPromises();await w.get('[data-testid="design-connect-codex"]').trigger('click');await flushPromises();expect(ipc).toHaveBeenCalledWith('design_mcp_configure',{client:'codex',remove:false});expect(w.text()).toContain('已配置 · 等待握手');expect(w.text()).not.toContain('最近调用');await w.get('[aria-label="移除 Codex / ChatGPT 桌面 连接"]').trigger('click');await flushPromises();expect(ipc).toHaveBeenCalledWith('design_mcp_configure',{client:'codex',remove:true});w.unmount();});
 it('shows a handshake independently of tool usage',async()=>{ipc.mockResolvedValue({...initial(),running:true,clients:[{id:'claude-desktop',configured:true,lastCall:null,lastHandshake:Date.now()}]});const w=mount(DesignMcpSettings);await flushPromises();expect(w.text()).toContain('握手成功 · 尚未调用');expect(w.text()).not.toContain('等待握手');w.unmount();});
 it('shows the web account steps instead of claiming localhost is connected to ChatGPT',async()=>{ipc.mockImplementation(async(command)=>command==='design_mcp_configure'?{...initial(),running:true,clients:[{id:'chatgpt-web',configured:true,lastCall:null}],tunnelCommand:'xiaoban --design-mcp-stdio'}:initial());const w=mount(DesignMcpSettings);await flushPromises();await w.get('[data-testid="design-connect-chatgpt-web"]').trigger('click');await flushPromises();expect(w.get('[data-testid="design-web-guide"]').text()).toContain('选择 Tunnel');expect((w.get('textarea').element as HTMLTextAreaElement).value).toBe('xiaoban --design-mcp-stdio');w.unmount();});
 it('surfaces registration errors without announcing success',async()=>{ipc.mockImplementation(async(command)=>{if(command==='design_mcp_configure')throw new Error('配置无法写入');return initial()});const w=mount(DesignMcpSettings);await flushPromises();await w.get('[data-testid="design-connect-claude-code"]').trigger('click');await flushPromises();expect(w.get('[role="alert"]').text()).toContain('配置无法写入');expect(w.text()).not.toContain('配置已保存');w.unmount();});
});
