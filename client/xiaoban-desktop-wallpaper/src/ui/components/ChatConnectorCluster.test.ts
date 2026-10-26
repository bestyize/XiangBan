// @vitest-environment happy-dom
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '../../bridge.ts';
import ChatConnectorCluster from './ChatConnectorCluster.vue';

vi.mock('../../bridge.ts', async original => ({ ...await original<typeof import('../../bridge.ts')>(), invoke: vi.fn() }));
const status = [
  { id: 'feishu', name: '飞书', connected: true, enabled: true, toolCount: 7 },
  { id: 'github', name: 'GitHub', connected: true, enabled: true, toolCount: 3 },
  { id: 'notion', name: 'Notion', connected: false, enabled: true, toolCount: 0 },
  { id: 'old', name: '旧服务', connected: false, enabled: false, toolCount: 0 },
];
const pick = (id: string, name: string) => ({ id, name });
let wrapper: VueWrapper;
function open(selected = [pick('feishu', '飞书')], extra: Record<string, unknown> = {}) {
  wrapper = mount(ChatConnectorCluster, { attachTo: document.body, props: { selected, desktop: true, ...extra } });
  return wrapper;
}
beforeEach(() => { vi.mocked(invoke).mockReset().mockImplementation(async command => (command === 'ai_store_connector_status' ? status : [])); });
afterEach(() => wrapper?.unmount());

describe('connector cluster', () => {
  it('shows nothing until a connector is chosen', () => {
    open([]);
    expect(wrapper.find('[data-testid=chat-connectors]').exists()).toBe(false);
  });
  it('overlaps at most three icons and folds the rest into +N', () => {
    open(['a', 'b', 'c', 'd', 'e'].map(id => pick(id, `服务${id}`)));
    const tiles = wrapper.findAll('.tiles .tile');
    expect(tiles).toHaveLength(4);
    expect(tiles[3].text()).toBe('+2');
    expect(wrapper.get('[data-testid=chat-connectors-toggle]').attributes('aria-label')).toBe('已选 5 个连接器，点击管理');
  });
  it('lists the installed connectors with a switch for the ready ones and a way to connect the rest', async () => {
    open(); await wrapper.get('[data-testid=chat-connectors-toggle]').trigger('click'); await flushPromises();
    expect(invoke).toHaveBeenCalledWith('ai_store_connector_status');
    expect(wrapper.get('[data-testid=chat-connector-feishu]').text()).toContain('已连接 · 7 个工具');
    expect(wrapper.get('[aria-label="优先使用 飞书"]').attributes('aria-checked')).toBe('true');
    expect(wrapper.get('[aria-label="优先使用 GitHub"]').attributes('aria-checked')).toBe('false');
    expect(wrapper.find('[aria-label="优先使用 Notion"]').exists()).toBe(false);
    expect(wrapper.get('[data-testid=chat-connector-notion]').text()).toContain('尚未连接');
    expect(wrapper.get('[data-testid=chat-connector-old]').text()).toContain('已停用');
  });
  it('reports a switch and a request to connect, and closes on Escape or an outside click', async () => {
    open(); await wrapper.get('[data-testid=chat-connectors-toggle]').trigger('click'); await flushPromises();
    await wrapper.get('[aria-label="优先使用 GitHub"]').trigger('click');
    expect(wrapper.emitted('toggle')).toEqual([[{ id: 'github', name: 'GitHub' }]]);
    await wrapper.get('[data-testid=chat-connector-notion] .row-link').trigger('click');
    expect(wrapper.emitted('manage')).toHaveLength(1);
    expect(wrapper.find('[role=dialog]').exists()).toBe(false);
    await wrapper.get('[data-testid=chat-connectors-toggle]').trigger('click'); await flushPromises();
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' })); await flushPromises();
    expect(wrapper.find('[role=dialog]').exists()).toBe(false);
    await wrapper.get('[data-testid=chat-connectors-toggle]').trigger('click'); await flushPromises();
    document.body.dispatchEvent(new Event('pointerdown', { bubbles: true })); await flushPromises();
    expect(wrapper.find('[role=dialog]').exists()).toBe(false);
  });
  it('keeps a chosen connector that is no longer installed so it can still be dropped', async () => {
    open([pick('gone', '已删除的服务')]); await wrapper.get('[data-testid=chat-connectors-toggle]').trigger('click'); await flushPromises();
    expect(wrapper.get('[data-testid=chat-connector-gone]').text()).toContain('已不可用');
    expect(wrapper.get('[aria-label="优先使用 已删除的服务"]').attributes('aria-checked')).toBe('true');
  });
  it('says so when the list cannot be read, and opens upward or downward as asked', async () => {
    vi.mocked(invoke).mockRejectedValue('boom');
    open([pick('feishu', '飞书')], { down: true });
    await wrapper.get('[data-testid=chat-connectors-toggle]').trigger('click'); await flushPromises();
    expect(wrapper.get('[role=status]').text()).toContain('读取连接器失败');
    expect(wrapper.get('.cluster-pop').classes()).toContain('down');
  });
});
