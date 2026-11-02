// @vitest-environment happy-dom
import { beforeEach, expect, it, vi } from 'vitest';
import { invoke } from '../../bridge.ts';
import { activeSkinId, applySkin, refreshSkins } from './store.ts';
vi.mock('../../bridge.ts', () => ({ invoke: vi.fn() }));
beforeEach(() => { vi.mocked(invoke).mockReset(); activeSkinId.value = null; });

it('does not let an older list response undo a skin change', async () => {
  let finish!: (state: unknown) => void;
  vi.mocked(invoke).mockReturnValueOnce(new Promise(resolve => { finish = resolve; }));
  const old = refreshSkins();
  vi.mocked(invoke).mockResolvedValueOnce({ skins: [], activeId: 'builtin:jiangnan' });
  await applySkin('builtin:jiangnan');
  finish({ skins: [], activeId: null }); await old;
  expect(activeSkinId.value).toBe('builtin:jiangnan');
});

it('uses the latest host state when an Agent notification overlaps a UI action', async () => {
  let finish!: (state: unknown) => void;
  vi.mocked(invoke).mockReturnValueOnce(new Promise(resolve => { finish = resolve; }));
  const applying = applySkin('builtin:jiangnan');
  vi.mocked(invoke).mockResolvedValueOnce({ skins: [], activeId: 'builtin:seaside-train' });
  await refreshSkins();
  finish({ skins: [], activeId: 'builtin:jiangnan' }); await applying;
  expect(activeSkinId.value).toBe('builtin:seaside-train');
});
