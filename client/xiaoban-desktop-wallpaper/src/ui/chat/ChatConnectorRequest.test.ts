// @vitest-environment happy-dom
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '../../bridge.ts';
import ChatConnectorRequest from './ChatConnectorRequest.vue';
import type { QuestionRequest } from './types.ts';
import type { InstalledConnector } from '../ai-store/connector-plugins.ts';

vi.mock('../../bridge.ts', () => ({ invoke: vi.fn() }));
vi.mock('naive-ui', async () => {
  const { defineComponent } = await import('vue');
  return {
    NModal: defineComponent({ props: ['show'], template: '<section v-if="show" role="dialog"><slot /></section>' }),
    NAlert: defineComponent({ template: '<div role="alert"><slot /></div>' }),
  };
});
const connector: InstalledConnector = { id: 'docs:main', plugin: 'docs', namespace: 'acme', name: '文档服务', description: '查找文档', version: '1', source: 'market', iconUrl: null, auth: { type: 'auto', configured: [] }, status: 'disconnected', enabled: false, tools: [], examples: [], skills: [] };
const request: QuestionRequest = { id: 'q1', session: 's1', question: '连接服务后继续当前任务', options: [], connectors: [{ reference: 'installed:docs:main', name: '文档服务', description: '查找文档', installed: connector }] };
let wrapper: VueWrapper;
beforeEach(() => {
  vi.mocked(invoke).mockReset().mockImplementation(async command => {
    if (command === 'ai_connector_installed') return [connector];
    if (command === 'ai_connector_configure') return { ...connector, status: 'connected' };
    return null;
  });
});
afterEach(() => { wrapper?.unmount(); });
describe('conversational connector request', () => {
  it('connects an installed service and answers the pending tool to resume its run', async () => {
    wrapper = mount(ChatConnectorRequest, { props: { request } });
    expect(invoke).not.toHaveBeenCalled();
    await wrapper.get('.connector-choice').trigger('click'); await flushPromises();
    expect(wrapper.get('[role=dialog]').text()).toContain('无需手动填写配置');
    await wrapper.get('.store-primary').trigger('click'); await flushPromises();
    expect(wrapper.emitted('answer')).toEqual([['installed:docs:main']]);
  });
  it('can skip without installing, connecting or exposing credentials', async () => {
    wrapper = mount(ChatConnectorRequest, { props: { request } });
    await wrapper.findAll('button').find(b => b.text() === '暂不连接')!.trigger('click');
    expect(wrapper.emitted('answer')).toEqual([[null]]); expect(invoke).not.toHaveBeenCalled();
  });
  it('checks live state and lets an already connected service continue', async () => {
    vi.mocked(invoke).mockResolvedValueOnce([{ ...connector, status: 'connected' }]);
    wrapper = mount(ChatConnectorRequest, { props: { request } });
    await wrapper.get('.connector-choice').trigger('click'); await flushPromises();
    await wrapper.findAll('button').find(b => b.text() === '继续当前任务')!.trigger('click');
    expect(wrapper.emitted('answer')).toEqual([['installed:docs:main']]);
    expect(invoke).not.toHaveBeenCalledWith('ai_connector_configure', expect.anything());
  });
  it('keeps market installation mounted throughout install, connection and OAuth', async () => {
    const marketRequest: QuestionRequest = { ...request, connectors: [{ reference: 'market:acme/docs', name: '文档服务', description: '查找文档', namespace: 'acme', slug: 'docs', version: '1', connector: { displayName: { zh: '文档服务' }, displayDescription: {}, examples: {}, authTypes: ['none'], tokenFields: [], transports: ['streamable-http'], skills: [], skillCount: 0, disabledToolCount: 0 } }] };
    let installed = false;
    vi.mocked(invoke).mockImplementation(async command => {
      if (command === 'ai_connector_installed') return installed ? [connector] : [];
      if (command === 'ai_market_inspect') return { sha256: 'a'.repeat(64), manifest: { version: '1', servers: [], skills: [] }, scripts: [] };
      if (command === 'ai_market_install') { installed = true; return; }
      if (command === 'ai_connector_configure') return { status: 'authorization_required', flowId: 'f1', authorizationUrl: 'https://auth.example/login' };
      return null;
    });
    wrapper = mount(ChatConnectorRequest, { props: { request: marketRequest } });
    await wrapper.get('.connector-choice').trigger('click'); await flushPromises();
    await wrapper.get('.store-primary').trigger('click'); await flushPromises();
    await wrapper.get('.review-confirm input').setValue(true); await wrapper.get('.store-primary').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('ai_market_install', expect.anything());
    expect(wrapper.text()).toContain('请在浏览器完成授权');
    expect(invoke).toHaveBeenCalledWith('open_download_url', { url: 'https://auth.example/login' });
    expect(invoke).not.toHaveBeenCalledWith('ai_store_connector_auth_cancel', expect.anything());
    expect(wrapper.emitted('answer')).toBeUndefined();
    wrapper.unmount(); expect(invoke).toHaveBeenCalledWith('ai_store_connector_auth_cancel', { flowId: 'f1' });
  });
});
