// @vitest-environment happy-dom
import { flushPromises, shallowMount } from '@vue/test-utils';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { NSelect } from 'naive-ui';
import CarePage from '../pages/CarePage.vue';
import { invoke } from '../../bridge';
import { emptyCareStatus } from './state';
import type { ControlStore } from '../control-types';
vi.mock('../../bridge', () => ({ invoke: vi.fn(), listen: vi.fn(async () => () => {}) }));

const shanghai = { name: '上海', country: '中国', admin1: '', latitude: 31.22, longitude: 121.45 };
const beijing = { name: '北京', country: '中国', admin1: '', latitude: 39.9, longitude: 116.4 };
let located: typeof shanghai;

async function mountNative(provider: 'open-meteo' | 'disabled' = 'open-meteo') {
  const status = emptyCareStatus();
  status.config.weatherSource.provider = provider;
  vi.mocked(invoke).mockImplementation(async (command: string) => {
    if (command === 'get_care_settings') return status;
    if (command === 'detect_care_city') return located;
    throw new Error(`unexpected command ${command}`);
  });
  const store = { desktop: true, settings: { companionEnabled: true }, notify: vi.fn(), navigate: vi.fn() } as unknown as ControlStore;
  const wrapper = shallowMount(CarePage, { props: { store }, global: { renderStubDefaultSlot: true, stubs: { NModal: { props: ['show'], template: '<div v-if="show"><slot /></div>' } } } });
  await flushPromises();
  return { wrapper, store };
}
const detectCalls = () => vi.mocked(invoke).mock.calls.filter(([command]) => command === 'detect_care_city').length;
type Wrapper = Awaited<ReturnType<typeof mountNative>>['wrapper'];
// The page's own open(): with Naive UI replaced by stubs, a click on the stubbed button does not run it.
const openDialog = (wrapper: Wrapper, kind: string) => (wrapper.vm as unknown as { open: (kind: string) => void }).open(kind);
async function chooseWeather(wrapper: Wrapper) {
  openDialog(wrapper, 'custom'); await flushPromises();
  const kind = wrapper.findAllComponents(NSelect).find(item => item.attributes('data-testid') === 'care-kind')!;
  kind.vm.$emit('update:value', 'weather');
  await flushPromises();
}

describe('weather reminder city pre-fill', () => {
  beforeEach(() => { vi.mocked(invoke).mockReset(); located = shanghai; });

  it('fills the city from the network location when a weather reminder is started', async () => {
    const { wrapper } = await mountNative();
    expect(detectCalls()).toBe(0);
    await chooseWeather(wrapper);
    expect(detectCalls()).toBe(1);
    expect(wrapper.get('[data-testid="care-city-state"]').text()).toContain('上海');
    expect(wrapper.find('[data-testid="care-city-located"]').exists()).toBe(true);
    wrapper.unmount();
  });

  it('makes no location request while the weather source is off', async () => {
    const { wrapper } = await mountNative('disabled');
    openDialog(wrapper, 'weather'); await flushPromises();
    expect(detectCalls()).toBe(0);
    wrapper.unmount();
  });

  it('starts locating as soon as the weather dialog is opened directly', async () => {
    const { wrapper } = await mountNative();
    openDialog(wrapper, 'weather'); await flushPromises();
    expect(detectCalls()).toBe(1);
    expect(wrapper.get('[data-testid="care-city-state"]').text()).toContain('上海');
    wrapper.unmount();
  });

  it('asking again by hand replaces the earlier guess', async () => {
    const { wrapper } = await mountNative();
    await chooseWeather(wrapper);
    located = beijing;
    await (wrapper.vm as unknown as { prefillCity: (manual: boolean) => Promise<void> }).prefillCity(true);
    await flushPromises();
    expect(detectCalls()).toBe(2);
    expect(wrapper.get('[data-testid="care-city-state"]').text()).toContain('北京');
    wrapper.unmount();
  });

  it('keeps the form usable when locating fails', async () => {
    const { wrapper } = await mountNative();
    vi.mocked(invoke).mockImplementation(async (command: string) => {
      if (command === 'get_care_settings') return emptyCareStatus();
      throw new Error('暂时无法根据网络位置判断城市');
    });
    await chooseWeather(wrapper);
    expect(wrapper.get('[data-testid="care-city-state"]').text()).toBe('尚未选择城市');
    wrapper.unmount();
  });
});
