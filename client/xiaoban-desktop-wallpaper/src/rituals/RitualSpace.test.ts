// @vitest-environment happy-dom
import { mount, flushPromises } from '@vue/test-utils';
import { beforeEach, afterEach, describe, expect, it, vi } from 'vitest';
import RitualSpace from './RitualSpace.vue';
import { emptyJournal, parseJournal, fortunes } from './journal.ts';
const storage = vi.hoisted(() => ({ load: vi.fn(), save: vi.fn() }));
vi.mock('./journal.ts', async original => ({ ...await original<typeof import('./journal.ts')>(), loadJournal: storage.load, saveJournal: storage.save }));
vi.mock('./PondScene.vue', () => ({ default: { template: '<div />' } }));
vi.mock('./FortuneVessel.vue', () => ({ default: { template: '<div />' } }));
beforeEach(() => {
  vi.useFakeTimers(); storage.load.mockReset().mockResolvedValue(emptyJournal()); storage.save.mockReset().mockResolvedValue(undefined);
  window.history.replaceState({}, '', '/?scene=ritual&ritual=fortune');
});
afterEach(() => { vi.useRealTimers(); });
describe('independent wish scenes', () => {
  it('prevents repeated draws during the ritual and collects the revealed fortune once', async () => {
    const view = mount(RitualSpace); await flushPromises();
    await view.get('[data-testid="draw-fortune"]').trigger('click');
    expect(view.get('[data-testid="draw-fortune"]').attributes('disabled')).toBeDefined();
    await vi.advanceTimersByTimeAsync(1900);
    await view.get('[data-testid="save-fortune"]').trigger('click'); await flushPromises();
    expect(storage.save).toHaveBeenCalledOnce();
    const saved = storage.save.mock.calls[0][0]; expect(saved.saved).toHaveLength(1); expect(fortunes[saved.saved[0]]).toBeDefined();
    expect(view.get('[data-testid="save-fortune"]').text()).toContain('已收藏'); view.unmount();
  });
  it('keeps a wish draft when persistence fails and publishes a lamp only after saving succeeds', async () => {
    window.history.replaceState({}, '', '/?scene=ritual&ritual=pond');
    storage.save.mockRejectedValueOnce(new Error('磁盘不可写'));
    const view = mount(RitualSpace); await flushPromises();
    await view.get('[data-testid="write-wish"]').trigger('click');
    await view.get('textarea').setValue('  愿你平安  ');
    await view.get('form').trigger('submit'); await flushPromises();
    expect(view.get('textarea').element.value).toBe('  愿你平安  ');
    expect(view.text()).toContain('已寄 0 份心愿'); expect(view.get('[role="alert"]').text()).toContain('磁盘不可写');
    await view.get('form').trigger('submit'); await flushPromises();
    expect(storage.save.mock.calls[1][0].wishes[0].text).toBe('愿你平安');
    expect(view.find('textarea').exists()).toBe(false); expect(view.text()).toContain('已寄 1 份心愿'); view.unmount();
  });
  it('does not overwrite a journal that failed to load', async () => {
    storage.load.mockRejectedValueOnce(new Error('记录损坏'));
    const view = mount(RitualSpace); await flushPromises();
    await view.get('[data-testid="draw-fortune"]').trigger('click'); await vi.advanceTimersByTimeAsync(1900);
    expect(view.get('[data-testid="save-fortune"]').attributes('disabled')).toBeDefined();
    expect(storage.save).not.toHaveBeenCalled(); view.unmount();
  });
  it('rejects oversized, invalid-position and invalid-fortune records', () => {
    expect(() => parseJournal({version:1,saved:[12],wishes:[]})).toThrow();
    expect(() => parseJournal({version:1,saved:[],wishes:[{id:'a',theme:'平安',text:'福'.repeat(61),x:.5,y:.5}]})).toThrow();
    expect(() => parseJournal({version:1,saved:[],wishes:[{id:'a',theme:'平安',text:'福',x:10,y:.5}]})).toThrow();
  });
});
