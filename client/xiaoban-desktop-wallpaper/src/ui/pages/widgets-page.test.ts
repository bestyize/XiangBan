// @vitest-environment happy-dom
import { mount, flushPromises } from '@vue/test-utils';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import WidgetsPage from './WidgetsPage.vue';
import { defaults } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
vi.mock('../../widgets/WidgetSurface.vue', () => ({ default: { template: '<div class="surface" />' } }));
vi.stubGlobal('ResizeObserver', class { observe() {} disconnect() {} });
function setup(settingsRequest?:{id:string}|null) {
  const store = {
    desktop: false, settings: JSON.parse(JSON.stringify(defaults)), game: { active: false }, info: { platform: 'macos' },
    library: { items: [] }, notify: vi.fn(), startCreation: vi.fn(),
  } as unknown as ControlStore;
  return { view: mount(WidgetsPage, { props: { store, settingsRequest }, attachTo: document.body }), store };
}
describe('widgets page', () => {
  beforeEach(() => { localStorage.clear(); document.body.innerHTML = ''; });
  it('shows one flow without tabs, adds from the library and adjusts in a dialog', async () => {
    const { view } = setup(); await flushPromises();
    expect(view.find('[role="tab"]').exists()).toBe(false);
    expect(view.find('[data-testid="desktop-widgets"]').exists()).toBe(false);
    expect(view.text()).not.toContain('互动场景');
    await view.get('[data-testid="add-calendar"]').trigger('click'); await flushPromises();
    const card = view.get('[data-testid="desktop-widgets"] button.desktop-card');
    expect(view.find('[data-testid="edit-widgets"]').exists()).toBe(false);
    expect(view.text()).not.toContain('在桌面上摆放');
    expect(view.text()).not.toContain('完成摆放');
    expect(view.text()).toContain('按住组件顶部拖动');
    expect(document.querySelector('[data-testid="widget-settings"]')).toBeNull();
    await card.trigger('click'); await flushPromises();
    expect(document.querySelector('[data-testid="widget-settings"]')).not.toBeNull();
    (document.querySelector('[data-testid="widget-settings-done"]') as HTMLElement).click(); await flushPromises();
    view.unmount();
  });
  it('waits for layout loading, targets the exact instance, and can reopen the same widget', async () => {
    localStorage.setItem('xiaoban.widgets.v1', JSON.stringify({ version: 1, widgets: [
      { id: 'noteone', kind: 'notes', theme: 'paper', scale: 1, x: .1, y: .1, enabled: true, city: null, note: { title: '第一张便签', mode: 'note', text: '第一份内容', items: [] } },
      { id: 'notetwo', kind: 'notes', theme: 'paper', scale: 1, x: .6, y: .1, enabled: true, city: null, note: { title: '第二张便签', mode: 'note', text: '第二份内容', items: [] } },
    ] }));
    const { view } = setup({ id: 'notetwo' }); await flushPromises();
    expect((document.querySelector('[data-testid="widget-settings"] input') as HTMLInputElement).value).toBe('第二张便签');
    expect(view.emitted('settingsOpened')).toHaveLength(1);
    await view.setProps({ settingsRequest: null });
    (document.querySelector('[data-testid="widget-settings-done"]') as HTMLElement).click(); await flushPromises();
    await view.setProps({ settingsRequest: { id: 'notetwo' } }); await flushPromises();
    expect(view.emitted('settingsOpened')).toHaveLength(2);
    expect((document.querySelector('[data-testid="widget-settings"] textarea') as HTMLTextAreaElement).value).toBe('第二份内容');
    await view.setProps({ settingsRequest: { id: 'noteone' } }); await flushPromises();
    expect((document.querySelector('[data-testid="widget-settings"] textarea') as HTMLTextAreaElement).value).toBe('第一份内容');
    view.unmount();
  });
  it('acknowledges a missing widget without opening an empty dialog', async () => {
    const { view, store } = setup({ id: 'removed' }); await flushPromises();
    expect(document.querySelector('[data-testid="widget-settings"]')).toBeNull();
    expect(store.notify).toHaveBeenCalledWith('这个组件已不在桌面上', 'info');
    expect(view.emitted('settingsOpened')).toHaveLength(1);
    view.unmount();
  });
  it('leaves an existing placement session without exposing the retired controls', async () => {
    localStorage.setItem('xiaoban.widgets.v1', JSON.stringify({ version: 1, widgets: [], editing: true }));
    const { view } = setup(); await flushPromises();
    expect(view.text()).not.toContain('完成摆放');
    expect(view.text()).not.toContain('组件已锁定功能');
    expect(JSON.parse(localStorage.getItem('xiaoban.widgets.v1')!).editing).not.toBe(true);
    expect(view.find('[data-testid="edit-widgets"]').exists()).toBe(false);
    view.unmount();
  });
  it('opens the settings dialog straight away for a weather widget, which needs a city', async () => {
    const { view } = setup(); await flushPromises();
    await view.get('[data-testid="add-weather"]').trigger('click'); await flushPromises();
    expect(document.querySelector('[data-testid="widget-settings"]')).not.toBeNull();
    expect(document.body.textContent).toContain('天气城市');
    view.unmount();
  });
});
