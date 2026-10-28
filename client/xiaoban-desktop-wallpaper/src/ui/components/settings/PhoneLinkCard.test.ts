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
  pairing: null,
  lastError: null,
  ...patch,
});

describe('PhoneLinkCard', () => {
  beforeEach(() => {
    ipc.mockReset();
  });

  it('loads status, starts pairing and renders QR SVG, and unpairs devices', async () => {
    let state = baseStatus({
      peers: [{ id: 'phn_1', name: '小明的小米手机', online: true }],
    });
    ipc.mockImplementation(async (command: string, args?: Record<string, unknown>) => {
      if (command === 'phone_link_status') return state;
      if (command === 'phone_link_set_enabled') {
        state = { ...state, enabled: Boolean(args?.enabled), connected: Boolean(args?.enabled) };
        return state;
      }
      if (command === 'phone_link_begin_pairing') {
        state = {
          ...state,
          enabled: true,
          connected: true,
          pairing: {
            qrText: 'xbl1:eyJyIjoiaHR0cHM6Ly9saW5rLnhpYW5nYmFuLnh5ei9hcGkvbGluayIsImQiOiJkc2tfMSIsImMiOiJBBkNELUVGR0giLCJrIjoiQ19wdWIiLCJzIjoiU19zZWNyZXQifQ',
            code: 'ABCD-EFGH',
            expiresAtMs: Date.now() + 240_000,
          },
        };
        return state;
      }
      if (command === 'phone_link_cancel_pairing') {
        state = { ...state, pairing: null };
        return state;
      }
      if (command === 'phone_link_unpair') {
        state = { ...state, peers: state.peers.filter(p => p.id !== args?.peerId) };
        return state;
      }
      throw new Error(`unexpected command: ${command}`);
    });

    const wrapper = mount(PhoneLinkCard);
    await flushPromises();

    expect(wrapper.find('[data-testid="phone-link-peer"]').text()).toContain('小明的小米手机');
    expect(wrapper.find('[data-testid="phone-link-peer"]').text()).toContain('在线');

    // 点击“连接手机”打开配对窗口并渲染二维码 SVG
    await wrapper.get('[data-testid="phone-link-pair"]').trigger('click');
    await flushPromises();
    expect(ipc).toHaveBeenCalledWith('phone_link_begin_pairing');
    const qrBox = wrapper.get('[data-testid="phone-link-qr"]');
    expect(qrBox.html()).toContain('<svg');
    expect(wrapper.get('[data-testid="phone-link-countdown"]').text()).toContain('二维码有效剩余');

    // 关闭二维码
    await wrapper.get('[data-testid="phone-link-cancel"]').trigger('click');
    await flushPromises();
    expect(wrapper.find('[data-testid="phone-link-qr"]').exists()).toBe(false);

    // 移除已配对设备
    await wrapper.get('[data-testid="phone-link-unpair"]').trigger('click');
    await flushPromises();
    expect(ipc).toHaveBeenCalledWith('phone_link_unpair', { peerId: 'phn_1' });
    expect(wrapper.find('[data-testid="phone-link-peer"]').exists()).toBe(false);
  });
});
