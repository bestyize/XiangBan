// @vitest-environment happy-dom
import { flushPromises, mount } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import { reactive } from 'vue';
import EffectsPage from './EffectsPage.vue';
import EffectPreview from '../components/EffectPreview.vue';
import { defaults } from '../../bridge.ts';
import { parsePack } from '../../effects/parse.ts';
import type { EffectPack, EffectsChoice } from '../../effects/types.ts';
import type { ImportOutcome } from '../../effects/catalog.ts';
import type { ControlStore } from '../control-types.ts';

const slider = vi.hoisted(() => ({ name: 'SliderStub', props: ['value'], emits: ['update:value'], template: '<input class="slider" type="range" />' }));
// The page imports Naive UI's slider directly, so it is replaced at the module, not through mount stubs.
vi.mock('naive-ui', async (original) => ({ ...(await original<typeof import('naive-ui')>()), NSlider: slider }));

const pack = (id: string, name: string, source: 'builtin' | 'user' = 'builtin'): EffectPack => {
  const parsed = parsePack({ schemaVersion: 1, id, name, description: `${name}的说明`, layers: [{ count: 5, shape: { kind: 'circle' }, colors: ['#fff'] }], source });
  if (!parsed.ok) throw new Error(parsed.error);
  return parsed.pack;
};
/** The card previews only; the inspector on the right has its own, always playing preview. */
const cards = (wrapper: ReturnType<typeof mount>) => wrapper.findAllComponents(EffectPreview).filter(preview => preview.element.closest('.effect-grid'));
const none: EffectsChoice = { selected: '', params: {} };

function setup(effects: EffectsChoice = none, packs = [pack('rain', '下雨'), pack('maple', '枫叶飘落'), pack('glitter', '星尘', 'user')], errors: string[] = [], imported: ImportOutcome = { status: 'imported', id: 'glitter', name: '星尘', file: 'glitter.json' }) {
  const sent: EffectsChoice[] = [];
  const store = reactive({
    settings: { ...JSON.parse(JSON.stringify(defaults)), effects: JSON.parse(JSON.stringify(effects)) },
    game: { active: false }, notify: vi.fn(), effectList: { packs, errors, directory: '/data/effects' },
    loadEffects: vi.fn(async () => {}), openEffectsDirectory: vi.fn(async () => {}), importEffectPack: vi.fn(async () => imported),
    setEffects: vi.fn(async (next: EffectsChoice) => { sent.push(JSON.parse(JSON.stringify(next))); store.settings.effects = JSON.parse(JSON.stringify(next)); }),
  }) as unknown as ControlStore;
  const wrapper = mount(EffectsPage, { props: { store }, global: { stubs: { EffectPreview: true } } });
  return { wrapper, store, sent };
}

describe('desktop effects page', () => {
  it('plays hovered cards temporarily without changing the desktop selection', async () => {
    const { wrapper, store } = setup({ selected: 'rain', params: {} });
    const playing = () => cards(wrapper).filter(preview => preview.props('playing')).map(preview => preview.props('pack').id);
    const maple = wrapper.get('[data-testid="effect-maple"]');
    await maple.trigger('mouseenter');
    expect(playing()).toEqual(['maple']);
    expect(store.setEffects).not.toHaveBeenCalled();
    expect(store.settings.effects.selected).toBe('rain');
    await maple.trigger('mouseleave');
    expect(playing()).toEqual([]);
    await wrapper.get('[data-testid="effect-rain"]').trigger('click');
    await maple.trigger('mouseenter');
    expect(playing()).toEqual(['maple']);
    await maple.trigger('mouseleave');
    expect(playing()).toEqual(['rain']);
    wrapper.unmount();
  });
  it('keeps saved selections static until clicked and plays only the clicked card', async () => {
    const { wrapper } = setup({ selected: 'rain', params: {} });
    const previews = () => cards(wrapper);
    const playing = () => previews().filter(preview => preview.props('playing')).map(preview => preview.props('pack').id);
    expect(previews()).toHaveLength(3);
    expect(playing()).toEqual([]);
    await wrapper.get('[data-testid="effect-rain"]').trigger('click');
    expect(playing()).toEqual(['rain']);
    await wrapper.get('[data-testid="effect-maple"]').trigger('click');
    expect(playing()).toEqual(['maple']);
    await wrapper.get('[data-testid="effect-none"]').trigger('click');
    expect(playing()).toEqual([]);
    wrapper.unmount();
    const reopened = setup({ selected: 'maple', params: {} });
    expect(cards(reopened.wrapper).every(preview => !preview.props('playing'))).toBe(true);
    reopened.wrapper.unmount();
  });
  it('shows all effects and filters custom effects without changing the desktop selection', async () => {
    const { wrapper, store } = setup();
    for (const id of ['none', 'rain', 'maple']) expect(wrapper.find(`[data-testid="effect-${id}"]`).exists(), id).toBe(true);
    expect(wrapper.get('[data-testid="effect-none"]').attributes('aria-pressed')).toBe('true');
    expect(wrapper.find('[data-testid="effect-glitter"]').exists()).toBe(true);
    expect(wrapper.get('[data-testid="effect-rain"]').text()).not.toContain('我的');
    expect(wrapper.find('[data-testid="effect-tuning"]').exists()).toBe(false);
    expect(wrapper.find('h1').exists()).toBe(false);
    expect(wrapper.findAll('.xb-seg button').map(button => button.text())).toEqual(['全部', '在桌面上 0', '我的']);
    expect(store.loadEffects).toHaveBeenCalledOnce();
    await wrapper.get('[data-testid="effects-filter-mine"]').trigger('click');await flushPromises();
    expect(wrapper.get('[data-testid="effect-glitter"]').text()).toContain('星尘');
    expect(wrapper.find('[data-testid="effect-rain"]').exists()).toBe(false);
    expect(store.setEffects).not.toHaveBeenCalled();
    wrapper.unmount();
  });
  it.each(['rain', 'glitter'])('shows only the selected %s effect on the desktop and can turn it off', async id => {
    const params = { [id]: { density: 1.5, speed: 1, size: 1 } };
    const { wrapper, store, sent } = setup({ selected: id, params });
    await wrapper.get('[data-testid="effects-filter-desk"]').trigger('click');
    expect(cards(wrapper).map(preview => preview.props('pack').id)).toEqual([id]);
    expect(wrapper.get('[data-testid="effects-filter-desk"]').text()).toContain('1');
    expect(store.setEffects).not.toHaveBeenCalled();
    await wrapper.get('[data-testid="effect-none"]').trigger('click'); await flushPromises();
    expect(sent[0]).toEqual({ selected: '', params });
    expect(wrapper.get('[data-testid="desktop-effects-empty"]').text()).toContain('当前没有使用特效');
    expect(cards(wrapper)).toHaveLength(0);
    await wrapper.get('[data-testid="desktop-effects-empty"] button').trigger('click');
    expect(wrapper.get('[data-testid="effects-filter-all"]').attributes('aria-pressed')).toBe('true');
    expect(cards(wrapper)).toHaveLength(3);
    wrapper.unmount();
  });
  it('applies an effect at once, without any wallpaper being involved', async () => {
    const { wrapper, store, sent } = setup();
    await wrapper.get('[data-testid="effect-maple"]').trigger('click'); await flushPromises();
    expect(store.setEffects).toHaveBeenCalledTimes(1);
    expect(sent[0].selected).toBe('maple');
    expect(wrapper.get('[data-testid="effect-maple"]').attributes('aria-pressed')).toBe('true');
    expect(wrapper.find('[data-testid="effect-tuning"]').exists()).toBe(true);
    wrapper.unmount();
  });
  it('each effect keeps its own settings when you switch, and "no effects" keeps them all', async () => {
    const start: EffectsChoice = { selected: 'rain', params: { rain: { density: 1.8, speed: 2, size: 1 }, maple: { density: 0.5, speed: 1, size: 1.5 } } };
    const { wrapper, sent } = setup(start);
    expect(wrapper.get('[data-testid="effect-density-value"]').text()).toBe('×1.8');
    await wrapper.get('[data-testid="effect-maple"]').trigger('click'); await flushPromises();
    expect(sent[0]).toEqual({ selected: 'maple', params: start.params });
    expect(wrapper.get('[data-testid="effect-density-value"]').text()).toBe('×0.5');
    expect(wrapper.get('[data-testid="effect-size-value"]').text()).toBe('×1.5');
    await wrapper.get('[data-testid="effect-none"]').trigger('click'); await flushPromises();
    expect(sent[1]).toEqual({ selected: '', params: start.params });
    expect(wrapper.find('[data-testid="effect-tuning"]').exists()).toBe(false);
    wrapper.unmount();
  });
  it('sends slider changes once they settle, and the last values win', async () => {
    vi.useFakeTimers();
    try {
      const { wrapper, sent } = setup({ selected: 'maple', params: {} });
      const [density, speed, size] = wrapper.findAllComponents({ name: 'SliderStub' });
      density.vm.$emit('update:value', 1.4); density.vm.$emit('update:value', 2.1); speed.vm.$emit('update:value', 2.5); size.vm.$emit('update:value', 1.6);
      await vi.advanceTimersByTimeAsync(400);
      expect(sent).toHaveLength(1);
      expect(sent[0]).toEqual({ selected: 'maple', params: { maple: { density: 2.1, speed: 2.5, size: 1.6 } } });
      wrapper.unmount();
    } finally { vi.useRealTimers(); }
  });
  it('says so when the chosen effect has gone, and lists the files that could not be used', () => {
    const gone = setup({ selected: 'removed', params: {} }, [pack('rain', '下雨')], ['broken.json：JSON 格式错误']);
    expect(gone.wrapper.get('[data-testid="effect-missing"]').text()).toContain('removed');
    expect(gone.wrapper.get('[data-testid="effect-errors"]').text()).toContain('broken.json');
    expect(gone.wrapper.get('[data-testid="effect-none"]').attributes('aria-pressed')).toBe('true');
    gone.wrapper.unmount();
    const fine = setup({ selected: 'rain', params: {} });
    expect(fine.wrapper.find('[data-testid="effect-missing"]').exists()).toBe(false);
    expect(fine.wrapper.find('[data-testid="effect-errors"]').exists()).toBe(false);
    fine.wrapper.unmount();
  });
  it('does not call an effect missing while the list is still loading', () => {
    const { wrapper } = setup({ selected: 'maple', params: {} }, []);
    expect(wrapper.find('[data-testid="effect-missing"]').exists()).toBe(false);
    wrapper.unmount();
  });
  it('imports a pack file, says which effect arrived, and reloads the list', async () => {
    const { wrapper, store } = setup();
    await wrapper.get('[data-testid="effects-import"]').trigger('click'); await flushPromises();
    expect(store.importEffectPack).toHaveBeenCalledOnce();
    expect(store.notify).toHaveBeenCalledWith('已导入特效「星尘」，可以选用了', 'success');
    expect(wrapper.get('[data-testid="effects-filter-mine"]').attributes('aria-pressed')).toBe('true');
    expect(cards(wrapper).map(preview => preview.props('pack').id)).toEqual(['glitter']);
    expect(store.loadEffects).toHaveBeenCalledTimes(1);
    wrapper.unmount();
  });
  it('says nothing was imported when the picker was closed or the clash was declined', async () => {
    const closed = setup(none, [], [], { status: 'cancelled' });
    await closed.wrapper.get('[data-testid="effects-import"]').trigger('click'); await flushPromises();
    expect(closed.store.notify).not.toHaveBeenCalled();
    closed.wrapper.unmount();
    const declined = setup(none, [], [], { status: 'declined', id: 'glitter', existing: 'glitter.json' });
    await declined.wrapper.get('[data-testid="effects-import"]').trigger('click'); await flushPromises();
    expect(declined.store.notify).toHaveBeenCalledWith('特效「glitter」已经存在，没有替换', 'info');
    declined.wrapper.unmount();
  });
  it('reports why an import failed instead of pretending it worked', async () => {
    const { wrapper, store } = setup();
    vi.mocked(store.importEffectPack).mockRejectedValueOnce(new Error('JSON 格式错误：expected value'));
    await wrapper.get('[data-testid="effects-import"]').trigger('click'); await flushPromises();
    expect(store.notify).toHaveBeenCalledWith('JSON 格式错误：expected value', 'error');
    wrapper.unmount();
  });
  it('reports a failure instead of pretending the effect was applied', async () => {
    const { wrapper, store } = setup();
    vi.mocked(store.setEffects).mockRejectedValueOnce(new Error('没有找到特效“rain”'));
    await wrapper.get('[data-testid="effect-rain"]').trigger('click'); await flushPromises();
    expect(store.notify).toHaveBeenCalledWith('没有找到特效“rain”', 'error');
    expect(wrapper.get('[data-testid="effect-none"]').attributes('aria-pressed')).toBe('true');
    wrapper.unmount();
  });
});

