// @vitest-environment happy-dom
import { flushPromises, mount } from '@vue/test-utils';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import PhoneLinkCard, { type PhoneLinkStatus } from './PhoneLinkCard.vue';
import { invoke } from '../../../bridge.ts';

vi.mock('../../../bridge.ts', async importOriginal => ({
  ...(await importOriginal<typeof import('../../../bridge.ts')>()),
  desktop: true,
  invoke: vi.fn(),
  listen: vi.fn(async () => () => {}),
}));

const ipc = vi.mocked(invoke);

const baseStatus = (patch: Partial<PhoneLinkStatus> = {}): PhoneLinkStatus => ({
  enabled: false,
  connected: false,
  deviceId: 'dsk_1',
  deviceName: '测试电脑',
  relayUrl: 'https://link.xiangban.xyz/api/link',
  loggedIn: true,
  peers: [],
  lastError: null,
  ...patch,
});

describe('PhoneLinkCard', () => {
  beforeEach(() => {
    ipc.mockReset();
  });

  it('shows same-account phones, toggles remote control and removes a phone', async () => {
    let state = baseStatus({ peers: [{ id: 'phn_1', name: '小明的小米手机', online: true }] });
    ipc.mockImplementation(async (command: string, args?: Record<string, unknown>) => {
      if (command === 'phone_link_status') return state;
      if (command === 'phone_link_set_enabled') {
        state = { ...state, enabled: Boolean(args?.enabled), connected: Boolean(args?.enabled) };
        return state;
      }
      if (command === 'phone_link_remove_peer') {
        state = { ...state, peers: state.peers.filter(p => p.id !== args?.peerId) };
        return state;
      }
      throw new Error(`unexpected command: ${command}`);
    });

    const wrapper = mount(PhoneLinkCard);
    await flushPromises();
    expect(wrapper.find('[data-testid="phone-link-peer"]').text()).toContain('小明的小米手机');
    expect(wrapper.find('[data-testid="phone-link-peer"]').text()).toContain('在线');
    // 没有配对、二维码这些东西了。
    expect(wrapper.find('[data-testid="phone-link-qr"]').exists()).toBe(false);
    expect(wrapper.find('[data-testid="phone-link-pair"]').exists()).toBe(false);

    await wrapper.get('[data-testid="phone-link-toggle"]').trigger('click');
    await flushPromises();
    expect(ipc).toHaveBeenCalledWith('phone_link_set_enabled', { enabled: true });

    await wrapper.get('[data-testid="phone-link-remove"]').trigger('click');
    await flushPromises();
    expect(ipc).toHaveBeenCalledWith('phone_link_remove_peer', { peerId: 'phn_1' });
    expect(wrapper.find('[data-testid="phone-link-peer"]').exists()).toBe(false);
    expect(wrapper.find('[data-testid="phone-link-empty"]').exists()).toBe(true);
  });

  it('shows an error returned by the desktop', async () => {
    ipc.mockImplementation(async (command: string) => {
      if (command === 'phone_link_status') return baseStatus({ lastError: '中转服务提示：rate_limited' });
      throw new Error('x');
    });
    const wrapper = mount(PhoneLinkCard);
    await flushPromises();
    expect(wrapper.text()).toContain('rate_limited');
  });
});
