// @vitest-environment happy-dom
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import WindowChrome from './WindowChrome.vue';
import { invoke, listen } from '../../bridge.ts';
vi.mock('../../bridge.ts', () => ({ desktop: true, invoke: vi.fn(), listen: vi.fn() }));
let wrapper: VueWrapper;
let frame = { maximized: false, fullscreen: false, focused: true };
beforeEach(() => { vi.resetAllMocks(); frame = { maximized: false, fullscreen: false, focused: true }; vi.mocked(listen).mockResolvedValue(() => {}); vi.mocked(invoke).mockImplementation(async () => ({ ...frame }) as never); });
afterEach(() => { wrapper?.unmount(); });
it('keeps close, minimize and expand together on the left and never drags from a button', async () => {
  wrapper = mount(WindowChrome, { props: { title: '设置', platform: 'macos' } }); await flushPromises();
  expect(wrapper.findAll('button').map(b => b.attributes('aria-label'))).toEqual(['关闭窗口','最小化','全屏']);
  vi.mocked(invoke).mockClear();
  await wrapper.get('[data-testid="window-close"]').trigger('mousedown', { button: 0, detail: 1 });
  expect(invoke).not.toHaveBeenCalled();
  await wrapper.get('[data-testid="window-close"]').trigger('click');
  expect(invoke).toHaveBeenCalledWith('configuration_window_close');
  await wrapper.get('[data-testid="window-minimize"]').trigger('click');
  expect(invoke).toHaveBeenCalledWith('configuration_window_action', { action: 'minimize' });
});
it('uses native dragging, system double-click behavior and macOS fullscreen/Option zoom', async () => {
  wrapper = mount(WindowChrome, { props: { platform: 'macos' } }); await flushPromises();
  await wrapper.trigger('mousedown', { button: 0, detail: 1 });
  expect(invoke).toHaveBeenCalledWith('configuration_window_action', { action: 'drag' });
  vi.mocked(invoke).mockClear();
  await wrapper.trigger('mousedown', { button: 0, detail: 2 }); expect(invoke).not.toHaveBeenCalled();
  await wrapper.trigger('dblclick');
  expect(invoke).toHaveBeenCalledWith('configuration_window_action', { action: 'double-click' });
  await wrapper.get('[data-testid="window-expand"]').trigger('click');
  expect(invoke).toHaveBeenCalledWith('configuration_window_action', { action: 'fullscreen' });
  await wrapper.get('[data-testid="window-expand"]').trigger('click', { altKey: true });
  expect(invoke).toHaveBeenLastCalledWith('configuration_window_action', { action: 'maximize' });
});
it('exits fullscreen with Escape without closing the editor or losing its draft', async () => {
  frame.fullscreen = true;
  wrapper = mount(WindowChrome, { props: { platform: 'macos' } }); await flushPromises();
  vi.mocked(invoke).mockClear();
  window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', cancelable: true })); await flushPromises();
  expect(invoke).toHaveBeenCalledWith('configuration_window_action', { action: 'fullscreen' });
  expect(invoke).not.toHaveBeenCalledWith('configuration_window_close');
});
it('offers Windows maximize and restore, with F11 fullscreen and Ctrl+W close', async () => {
  frame.maximized = true;
  wrapper = mount(WindowChrome, { props: { platform: 'windows' } }); await flushPromises();
  expect(wrapper.get('[data-testid="window-expand"]').attributes('aria-label')).toBe('还原窗口');
  await wrapper.get('[data-testid="window-expand"]').trigger('click');
  expect(invoke).toHaveBeenCalledWith('configuration_window_action', { action: 'maximize' });
  window.dispatchEvent(new KeyboardEvent('keydown', { key: 'F11', cancelable: true }));
  expect(invoke).toHaveBeenCalledWith('configuration_window_action', { action: 'fullscreen' });
  window.dispatchEvent(new KeyboardEvent('keydown', { key: 'w', ctrlKey: true, cancelable: true }));
  expect(invoke).toHaveBeenCalledWith('configuration_window_close');
});
