// @vitest-environment happy-dom
import { flushPromises } from '@vue/test-utils';
import { afterEach, expect, it, vi } from 'vitest';
import type { CareDelivery } from './care-model.ts';

const { invokeMock, listenMock } = vi.hoisted(() => ({ invokeMock: vi.fn(), listenMock: vi.fn() }));
vi.mock('./bridge.ts', () => ({ invoke: invokeMock, listen: listenMock }));
vi.mock('./ui/design-tokens', () => ({ applyUiTokens: () => {} }));
import { mountCareBubble } from './care-bubble.ts';

afterEach(() => { vi.unstubAllGlobals(); invokeMock.mockReset(); listenMock.mockReset(); document.body.innerHTML = ''; });

it('plays a voice reminder once and stops it when dismissed', async () => {
  const delivery = { id: 'reminder-1', items: [{ text: '记得喝水', preview: false }], voice: true, sound: false, displaySeconds: 30 } as CareDelivery;
  let onDelivery: (value: CareDelivery | null) => void = () => {};
  const pause = vi.fn();
  vi.stubGlobal('Audio', class { src = ''; onended: (() => void) | null = null; onerror: (() => void) | null = null; pause = pause; play = vi.fn().mockResolvedValue(undefined); constructor(_url: string) {} });
  vi.stubGlobal('URL', { ...URL, createObjectURL: vi.fn(() => 'blob:reminder'), revokeObjectURL: vi.fn() });
  listenMock.mockImplementation(async (_event, callback) => { onDelivery = callback; return () => {}; });
  invokeMock.mockImplementation(async command => {
    if (command === 'care_ready') return delivery;
    if (command === 'care_synthesize') return new Uint8Array([1, 2, 3]).buffer;
    if (command === 'dismiss_care') return;
    throw new Error(command);
  });
  document.body.innerHTML = '<div id="app"></div>';
  await mountCareBubble(); await flushPromises();
  expect(invokeMock).toHaveBeenCalledWith('care_synthesize', { id: delivery.id });
  onDelivery(delivery); await flushPromises();
  expect(invokeMock.mock.calls.filter(([command]) => command === 'care_synthesize')).toHaveLength(1);
  document.querySelector<HTMLButtonElement>('.care-close')!.click(); await flushPromises();
  expect(pause).toHaveBeenCalled();
});
