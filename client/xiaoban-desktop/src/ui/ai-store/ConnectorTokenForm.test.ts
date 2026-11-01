// @vitest-environment happy-dom
import { flushPromises, mount } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import { invoke } from '../../bridge.ts';
import ConnectorTokenForm from './ConnectorTokenForm.vue';
vi.mock('../../bridge.ts', () => ({ invoke: vi.fn().mockResolvedValue(undefined) }));
describe('provider credential setup', () => {
  it('opens the plugin help link without sending form values to the browser', async () => {
    const wrapper = mount(ConnectorTokenForm, { props: { configured: [], confirmationRequired: false, cancelable: true, schema: { title: '文档连接配置', docUrl: 'https://docs.example/settings/mcp', docLabel: '获取 API 密钥', fields: [{ key: 'TOKEN', label: '密钥', type: 'password', required: true }] } } });
    await wrapper.get('input').setValue('test-secret');
    await wrapper.get('.token-doc-link').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('open_download_url', { url: 'https://docs.example/settings/mcp' });
    await wrapper.get('form').trigger('submit');
    expect(wrapper.emitted('connect')).toEqual([[{ TOKEN: 'test-secret' }]]);
    expect(wrapper.get('input').element.value).toBe('');
    wrapper.unmount();
  });
  it('omits unsafe documentation links', () => {
    for (const docUrl of ['javascript:alert(1)', 'file:///private', 'https://user:password@example.com']) {
      const wrapper = mount(ConnectorTokenForm, { props: { configured: [], schema: { title: '', fields: [], docUrl } } });
      expect(wrapper.find('.token-doc-link').exists()).toBe(false); wrapper.unmount();
    }
  });
});
