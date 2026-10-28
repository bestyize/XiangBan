// @vitest-environment happy-dom
import { flushPromises, mount } from '@vue/test-utils';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { reactive } from 'vue';
import CommunityApplyDialog from './CommunityApplyDialog.vue';
import { invoke, listen } from '../../bridge.ts';
import { applyFlow, closeApply, type PackPreview } from './apply.ts';
import type { ControlStore } from '../control-types.ts';

vi.mock('../../bridge.ts', () => ({
  invoke: vi.fn(),
  listen: vi.fn(async () => () => {}),
}));

const ipc = vi.mocked(invoke);
const onListen = vi.mocked(listen);

const samplePreview: PackPreview = {
  token: 'tok-123',
  packId: 7,
  kind: 'effect',
  title: '繁星流动',
  version: '2.0.0',
  trustLevel: 0,
  canApply: true,
  size: 1024,
  sha256: 'a'.repeat(64),
  items: [{ title: 'effect', detail: '星光粒子' }],
  startsDisabled: false,
};

describe('CommunityApplyDialog', () => {
  let linkHandler: (() => void) | null = null;

  beforeEach(() => {
    ipc.mockReset();
    closeApply();
    linkHandler = null;
    onListen.mockImplementation(async (event: string, handler: (payload: unknown) => void) => {
      if (event === 'community-link') {
        linkHandler = handler as () => void;
      }
      return () => {};
    });
  });

  it('takes link on mount and opens apply confirmation dialog', async () => {
    ipc.mockImplementation(async (command: string) => {
      if (command === 'community_take_link') {
        return {
          action: 'apply',
          kind: 'effect',
          packId: 7,
          version: 2,
          sha256: 'a'.repeat(64),
        };
      }
      if (command === 'community_prepare') {
        return samplePreview;
      }
      if (command === 'community_track') return undefined;
      throw new Error(`unhandled: ${command}`);
    });

    const store = reactive({ desktop: true, navigate: vi.fn(), notify: vi.fn() }) as unknown as ControlStore;
    mount(CommunityApplyDialog, { props: { store } });
    await flushPromises();

    expect(ipc).toHaveBeenCalledWith('community_take_link');
    expect(ipc).toHaveBeenCalledWith('community_prepare', { packId: 7, version: 2, sha256: 'a'.repeat(64) });
    expect(applyFlow.state.phase).toBe('confirm');
  });

  it('listens for community-link event and takes incoming link', async () => {
    let linkAvailable: Record<string, unknown> | null = null;
    ipc.mockImplementation(async (command: string) => {
      if (command === 'community_take_link') {
        const item = linkAvailable;
        linkAvailable = null;
        return item;
      }
      if (command === 'community_prepare') {
        return samplePreview;
      }
      if (command === 'community_track') return undefined;
      return undefined;
    });

    const store = reactive({ desktop: true, navigate: vi.fn(), notify: vi.fn() }) as unknown as ControlStore;
    mount(CommunityApplyDialog, { props: { store } });
    await flushPromises();
    expect(applyFlow.state.phase).toBe('idle');

    // 模拟新深链通过单实例转发到达
    linkAvailable = { action: 'apply', kind: 'effect', packId: 7, version: 2, sha256: 'a'.repeat(64) };
    expect(linkHandler).not.toBeNull();
    linkHandler?.();
    await flushPromises();

    expect(applyFlow.state.phase).toBe('confirm');
  });
});
