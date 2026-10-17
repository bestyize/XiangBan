// @vitest-environment happy-dom
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { defaults, wallpaperDefaults } from '../bridge.ts';
import { clone, createControlStore } from './control-store.ts';
import { invoke, listen } from '../bridge.ts';
vi.mock('../bridge.ts', async (importOriginal) => ({ ...await importOriginal<typeof import('../bridge.ts')>(), desktop: true, invoke: vi.fn(), listen: vi.fn() }));
const ipc = vi.mocked(invoke);
beforeEach(() => { vi.resetAllMocks(); vi.mocked(listen).mockResolvedValue(() => {}); });
describe('control state with native settings', () => {
  it('starts each creative action in a fresh chat with an editable prompt', async () => {
    const store = createControlStore(vi.fn());
    store.chat.currentId = 'existing';
    await store.startCreation('effect');
    expect(store.chat.currentId).toBeNull();
    expect(store.page).toBe('chat');
    expect(store.creationDraft).toBe('帮我创作一个桌面特效：');
    await store.startCreation('widget');
    expect(store.creationDraft).toBe('帮我创作一个桌面小组件：');
    await store.startCreation('game');
    expect(store.creationDraft).toBe('帮我创作一款小游戏：');
  });
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
  it('sends the chosen effect to the host and keeps the last acknowledged choice', async () => {
    const store = createControlStore(vi.fn());
    const choice = { selected: 'petals', params: { petals: { density: 1.6, speed: 1.5, size: 1.2 } } };
    ipc.mockImplementation(async (_command, args) => ({ ...clone(defaults), effects: args?.effects } as never));
    await store.setEffects(choice);
    expect(ipc).toHaveBeenCalledWith('set_desktop_effects', { effects: choice });
    expect(store.settings.effects).toEqual(choice);
    ipc.mockRejectedValueOnce(new Error('没有找到特效'));
    await expect(store.setEffects({ selected: 'gone', params: {} })).rejects.toThrow('没有找到特效');
    expect(store.settings.effects.selected).toBe('petals');
    ipc.mockImplementation(async (_command, args) => ({ ...clone(defaults), effects: args?.effects } as never));
    await store.setEffects({ selected: '', params: choice.params });
    expect(store.settings.effects.selected).toBe('');
  });
  it('sends the companion links switch to the host and keeps the acknowledged value', async () => {
    const store = createControlStore(vi.fn());
    expect(store.settings.companionLinks).toBe(true);
    ipc.mockImplementation(async (_command, args) => ({ ...clone(defaults), companionLinks: args?.enabled } as never));
    await store.setCompanionLinks(false);
    expect(ipc).toHaveBeenCalledWith('set_companion_links', { enabled: false });
    expect(store.settings.companionLinks).toBe(false);
    ipc.mockRejectedValueOnce(new Error('保存失败'));
    await expect(store.setCompanionLinks(true)).rejects.toThrow('保存失败');
    expect(store.settings.companionLinks).toBe(false);
  });
  it('sends the chosen shell to the host and keeps the acknowledged one', async () => {
    const store = createControlStore(vi.fn());
    expect(store.settings.companionSkin).toBe('cat');
    ipc.mockImplementation(async (_command, args) => ({ ...clone(defaults), companionSkin: args?.skin } as never));
    await store.setCompanionSkin('bunny');
    expect(ipc).toHaveBeenCalledWith('set_companion_skin', { skin: 'bunny' });
    expect(store.settings.companionSkin).toBe('bunny');
    ipc.mockRejectedValueOnce(new Error('没有这个外壳：x'));
    await expect(store.setCompanionSkin('x')).rejects.toThrow('没有这个外壳');
    expect(store.settings.companionSkin).toBe('bunny');
  });
  it('lists the effects through the host, keeping the usable ones and naming the rest', async () => {
    const store = createControlStore(vi.fn());
    const good = { schemaVersion: 1, id: 'glitter', name: '星尘', layers: [{ count: 5, shape: { kind: 'circle' }, colors: ['#fff'] }], source: 'user' };
    ipc.mockResolvedValue({ packs: [good, { id: 'bad' }], errors: ['x.json：JSON 格式错误'], directory: '/data/effects' } as never);
    await store.loadEffects();
    expect(ipc).toHaveBeenCalledWith('list_desktop_effects');
    expect(store.effectList.packs.map((pack) => pack.id)).toEqual(['glitter']);
    expect(store.effectList.packs[0].source).toBe('user');
    expect(store.effectList.errors).toHaveLength(2);
    expect(store.effectList.directory).toBe('/data/effects');
    await store.openEffectsDirectory();
    expect(ipc).toHaveBeenLastCalledWith('open_effects_directory');
  });
  it('imports a pack file, reloading the list only when one actually arrived', async () => {
    const store = createControlStore(vi.fn());
    ipc.mockImplementation(async (command) => command === 'import_effect_pack'
      ? { status: 'imported', id: 'glitter', name: '星尘', file: 'glitter.json' } as never
      : { packs: [], errors: [], directory: '/data/effects' } as never);
    await expect(store.importEffectPack()).resolves.toEqual({ status: 'imported', id: 'glitter', name: '星尘', file: 'glitter.json' });
    expect(ipc).toHaveBeenCalledWith('import_effect_pack');
    expect(ipc).toHaveBeenCalledWith('list_desktop_effects');

    ipc.mockClear();
    ipc.mockImplementation(async (command) => command === 'import_effect_pack' ? { status: 'cancelled' } as never : {} as never);
    await expect(store.importEffectPack()).resolves.toEqual({ status: 'cancelled' });
    expect(ipc).not.toHaveBeenCalledWith('list_desktop_effects');

    ipc.mockRejectedValueOnce(new Error('JSON 格式错误：expected value'));
    await expect(store.importEffectPack()).rejects.toThrow('JSON 格式错误');
  });
  it('does not adopt preview settings or reload the active library when opening a wallpaper preview', async () => {
    const store = createControlStore(vi.fn()); const before = clone(store.settings);
    ipc.mockResolvedValue({ ...before, wallpaperId: 'preview-item' } as never);
    await store.applyWallpaper('preview-item', clone(wallpaperDefaults), true);
    expect(clone(store.settings)).toEqual(before); expect(ipc).toHaveBeenCalledTimes(1);
  });
});
