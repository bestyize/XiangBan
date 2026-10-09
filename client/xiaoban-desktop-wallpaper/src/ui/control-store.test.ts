// @vitest-environment happy-dom
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { defaults, wallpaperDefaults } from '../bridge.ts';
import { clone, createControlStore } from './control-store.ts';
import { invoke, listen } from '../bridge.ts';
vi.mock('../bridge.ts', async (importOriginal) => ({ ...await importOriginal<typeof import('../bridge.ts')>(), desktop: true, invoke: vi.fn(), listen: vi.fn() }));
const ipc = vi.mocked(invoke);
beforeEach(() => { vi.resetAllMocks(); vi.mocked(listen).mockResolvedValue(() => {}); });
describe('control state with native settings', () => {
  it('does not replace loaded settings when the optional wallpaper directory fails', async () => {
    const native = { ...clone(defaults), size: 340, expression: 'proud' };
    ipc.mockImplementation(async (command) => {
      if (command === 'get_settings') return native as never;
      if (command === 'get_wallpaper_library') throw new Error('directory unavailable');
      if (command === 'get_app_info') return { platform: 'windows' } as never;
      if (command === 'get_startup_status') return { available: true, error: null } as never;
      if (command === 'get_game_status') return { active: false } as never;
      return { announcements: [], dismissed: {}, errors: [], update: null } as never;
    });
    const store = createControlStore(vi.fn()); await store.initialize();
    expect(store.settings.size).toBe(340); expect(store.settings.expression).toBe('proud'); expect(store.initialError).toBe('');
  });
  it('rebases queued appearance changes onto the last acknowledged native settings', async () => {
    const store = createControlStore(vi.fn());
    let release!: () => void;
    const gate = new Promise<void>(resolve => { release = resolve; });
    let count = 0;
    ipc.mockImplementation(async (_command, args) => { if (++count === 1) await gate; return { ...clone(defaults), ...args } as never; });
    const first = store.updateAppearance({ size: 360 });
    const second = store.updateAppearance({ expression: 'happy' });
    await Promise.resolve(); release(); await Promise.all([first, second]);
    expect(store.settings.size).toBe(360); expect(store.settings.expression).toBe('happy');
    expect(ipc.mock.calls[1][1]?.size).toBe(360);
  });
  it('does not adopt preview settings or reload the active library when opening a wallpaper preview', async () => {
    const store = createControlStore(vi.fn()); const before = clone(store.settings);
    ipc.mockResolvedValue({ ...before, wallpaperId: 'preview-item' } as never);
    await store.applyWallpaper('preview-item', clone(wallpaperDefaults), true);
    expect(clone(store.settings)).toEqual(before); expect(ipc).toHaveBeenCalledTimes(1);
  });
});
