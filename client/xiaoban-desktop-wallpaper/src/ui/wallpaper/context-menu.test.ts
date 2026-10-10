// @vitest-environment happy-dom
import { afterEach, expect, it, vi } from 'vitest';
import { createWallpaperContextMenu, probeWallpaperNativeMenu } from './context-menu.ts';
afterEach(() => vi.unstubAllGlobals());
it('uses actual native menu actions and releases resources across popup and disposal', async () => {
  const close = vi.fn().mockResolvedValue(undefined), popup = vi.fn().mockResolvedValue(undefined);
  const create = vi.fn().mockResolvedValue({ close, popup });
  vi.stubGlobal('window', { __TAURI__: { menu: { Menu: { new: create } } } });
  const context = createWallpaperContextMenu(), action = vi.fn();
  await context.show([{ key: 'apply', label: '应用到桌面' }], action);
  expect(popup).toHaveBeenCalledOnce(); expect(action).not.toHaveBeenCalled();
  create.mock.calls[0][0].items[0].action(); expect(action).toHaveBeenCalledWith('apply');
  await context.show([{ key: 'save', label: '保存' }], action); expect(close).toHaveBeenCalledOnce();
  context.dispose(); expect(close).toHaveBeenCalledTimes(2);
  create.mock.calls[1][0].items[0].action(); expect(action).toHaveBeenCalledTimes(1);
});
it('probes capabilities without showing a popup and falls back only without the native API', async () => {
  const close = vi.fn().mockResolvedValue(undefined), popup = vi.fn();
  vi.stubGlobal('window', { __TAURI__: { menu: { Menu: { new: vi.fn().mockResolvedValue({ close, popup }) } } } });
  expect(await probeWallpaperNativeMenu()).toBe(true); expect(close).toHaveBeenCalledOnce(); expect(popup).not.toHaveBeenCalled();
  vi.stubGlobal('window', {});
  expect(await createWallpaperContextMenu().show([], vi.fn())).toBe(false);
});
