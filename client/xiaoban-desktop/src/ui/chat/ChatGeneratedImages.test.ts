// @vitest-environment happy-dom
import { flushPromises, mount } from '@vue/test-utils';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '../../bridge.ts';
import ChatGeneratedImages from './ChatGeneratedImages.vue';
import type { ToolCall } from './replies.ts';

vi.mock('../../bridge.ts', () => ({ invoke: vi.fn() }));
const image = { id: 'a'.repeat(64) + '.png', name: '小伴生图.png', mime: 'image/png', width: 1024, height: 1024 };
const call = (status: ToolCall['status'] = 'done'): ToolCall => ({ kind: 'tool', id: 'g1', name: 'image_generate', arguments: '{}', output: '', status, ...(status === 'done' ? { generatedImages: [image] } : {}) });
beforeEach(() => { vi.mocked(invoke).mockReset(); vi.mocked(invoke).mockResolvedValue('data:image/png;base64,YWJj'); });

describe('image generation delivery', () => {
  it('shows progress and renders the completed gallery outside execution logs', async () => {
    const wrapper = mount(ChatGeneratedImages, { props: { calls: [call('running')] } });
    expect(wrapper.get('[role=status]').text()).toContain('正在生成图片');
    expect(invoke).not.toHaveBeenCalled();
    await wrapper.setProps({ awaitingApproval: true });
    expect(wrapper.get('[role=status]').text()).toContain('等待生图确认');
    await wrapper.setProps({ calls: [call()] }); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('chat_generated_image_read', { id: image.id });
    expect(wrapper.get('img').attributes('src')).toBe('data:image/png;base64,YWJj');
    expect(wrapper.find('[role=status]').exists()).toBe(false);
    await wrapper.setProps({ calls: [{ ...call(), output: '模型完成了文字回复' }] }); await flushPromises();
    expect(invoke).toHaveBeenCalledTimes(1);
    wrapper.unmount();
  });
  it('opens the original natively and only reports saved after the save dialog succeeds', async () => {
    const wrapper = mount(ChatGeneratedImages, { props: { calls: [call()] } }); await flushPromises();
    await wrapper.get('.image-preview').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('chat_generated_image_open', { id: image.id });
    vi.mocked(invoke).mockResolvedValueOnce(false);
    await wrapper.get('figcaption button:last-child').trigger('click'); await flushPromises();
    expect(wrapper.text()).not.toContain('已保存');
    vi.mocked(invoke).mockResolvedValueOnce(true);
    await wrapper.get('figcaption button:last-child').trigger('click'); await flushPromises();
    expect(wrapper.text()).toContain('已保存');
    expect(invoke).toHaveBeenCalledWith('chat_generated_image_save', { id: image.id });
    wrapper.unmount();
  });
  it('can reread a missing preview without ever sending a new generation request', async () => {
    vi.mocked(invoke).mockRejectedValueOnce(Error('图片预览读取失败'));
    const wrapper = mount(ChatGeneratedImages, { props: { calls: [call()] } }); await flushPromises();
    expect(wrapper.get('[role=alert]').text()).toContain('读取失败');
    vi.mocked(invoke).mockResolvedValueOnce(false);
    await wrapper.get('figcaption button:last-child').trigger('click'); await flushPromises();
    expect(wrapper.get('[role=alert]').text()).toContain('读取失败');
    await wrapper.get('.image-error button').trigger('click'); await flushPromises();
    expect(wrapper.find('img').exists()).toBe(true);
    expect(vi.mocked(invoke).mock.calls.map(([name]) => name)).toEqual(['chat_generated_image_read', 'chat_generated_image_save', 'chat_generated_image_read']);
    wrapper.unmount();
  });
  it('keeps failed generations visible and ignores arbitrary remote preview URLs', async () => {
    const wrapper = mount(ChatGeneratedImages, { props: { calls: [{ ...call('error'), output: '需要先兑换付费额度' }] } });
    expect(wrapper.get('[role=alert]').text()).toContain('兑换付费额度');
    vi.mocked(invoke).mockResolvedValueOnce('https://untrusted.example/a.png');
    await wrapper.setProps({ calls: [call()] }); await flushPromises();
    expect(wrapper.find('img').exists()).toBe(false);
    expect(wrapper.get('[role=alert]').text()).toContain('无法识别');
    wrapper.unmount();
  });
});
