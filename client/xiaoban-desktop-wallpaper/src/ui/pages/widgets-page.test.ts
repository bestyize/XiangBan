// @vitest-environment happy-dom
import { mount, flushPromises } from '@vue/test-utils';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import WidgetsPage from './WidgetsPage.vue';
import { defaults } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
vi.mock('../../widgets/WidgetSurface.vue', () => ({ default: { template: '<div class="surface" />' } }));
vi.stubGlobal('ResizeObserver', class { observe() {} disconnect() {} });
function setup() {
  const store = {
    desktop: false, settings: JSON.parse(JSON.stringify(defaults)), game: { active: false }, info: { platform: 'macos' },
    library: { items: [] }, notify: vi.fn(), startCreation: vi.fn(),
  } as unknown as ControlStore;
  return { view: mount(WidgetsPage, { props: { store }, attachTo: document.body }), store };
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
    expect(document.querySelector('[data-testid="widget-settings"]')).toBeNull();
    await card.trigger('click'); await flushPromises();
    expect(document.querySelector('[data-testid="widget-settings"]')).not.toBeNull();
    (document.querySelector('[data-testid="widget-settings-done"]') as HTMLElement).click(); await flushPromises();
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
