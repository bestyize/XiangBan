// @vitest-environment happy-dom
import { describe, expect, it, vi } from 'vitest';
import { createCareState, emptyCareStatus } from './state';
import { newCareRule } from '../../care-model';
vi.mock('../../bridge', () => ({ invoke: vi.fn(), listen: vi.fn(async () => () => {}) }));
describe('care editor state', () => {
  it('keeps an editing draft and unsaved preferences through background events', () => {
    const care = createCareState(false, vi.fn());
    care.open(); care.draft.value.text = '正在编辑'; care.preferences.sound = true; care.preferencesDirty.value = true;
    const incoming = emptyCareStatus(); incoming.rules = [{ ...newCareRule(), id: 'native', text: '后台规则' }];
    care.receive(incoming);
    expect(care.status.value.rules[0]?.text).toBe('后台规则');
    expect(care.draft.value.text).toBe('正在编辑'); expect(care.preferences.sound).toBe(true);
  });
  it('uses Open-Meteo by default', () => {
    expect(emptyCareStatus().config.weatherSource.provider).toBe('open-meteo');
  });
  it('starts empty and refuses weather rules while source is disabled', async () => {
    const notify = vi.fn(); const care = createCareState(false, notify);
    await care.initialize(); expect(care.status.value.rules).toEqual([]);
    care.status.value.config.weatherSource.provider = 'disabled';
    care.open('weather'); await care.save();
    expect(care.status.value.rules).toEqual([]); expect(notify).toHaveBeenCalledWith(expect.stringContaining('天气来源'), 'error');
  });
  it('cancel does not save; invalid custom text does not close editor', async () => {
    const care = createCareState(false, vi.fn()); care.open(); care.draft.value.text = '取消的内容'; care.editing.value = false;
    expect(care.status.value.rules).toHaveLength(0);
    care.open(); care.draft.value.text = ' '; await care.save(); expect(care.editing.value).toBe(true); expect(care.status.value.rules).toHaveLength(0);
    care.draft.value.text = '喝水'; await care.save(); expect(care.status.value.rules[0]?.text).toBe('喝水'); expect(care.editing.value).toBe(false);
  });
});
