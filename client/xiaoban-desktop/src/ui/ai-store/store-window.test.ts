// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '../../bridge.ts';
import { acceptStoreTask, openStoreWindow } from './store-window.ts';
import type { ControlStore } from '../control-types.ts';
vi.mock('../../bridge.ts', () => ({ invoke: vi.fn() }));
const ipc = vi.mocked(invoke);
beforeEach(() => { ipc.mockReset(); ipc.mockResolvedValue(null); });
afterEach(() => { vi.unstubAllGlobals(); });
function mockStore() { return { chat: { select: vi.fn(), pendingExpert: undefined }, creationDraft: '', navigate: vi.fn() } as unknown as ControlStore; }
describe('native plugin details', () => {
  it('passes only public identity to window creation, excluding config and credentials', async () => {
    const item = { slug: 'docs', name: '文档', description: '', token: 'secret', config: { headers: { Authorization: 'private' } } };
    await openStoreWindow({ view: 'connector', kind: 'connectors', source: 'market', item });
    const args = ipc.mock.calls[0][1]!;
    expect(args.target).toBe('store');
    expect(JSON.parse(args.provider as string)).toEqual({ view: 'connector', kind: 'connectors', source: 'market', item: { slug: 'docs', name: '文档', description: '' } });
    expect(args.provider).not.toContain('secret'); expect(args.provider).not.toContain('Authorization');
  });
  it('resolves expert instructions from the installed record rather than the requesting window', async () => {
    ipc.mockResolvedValue([{ slug: 'writer', name: '写作专家', description: '', plugin: true, agent: 'installed-persona' }]);
    const store = mockStore();
    await acceptStoreTask(store, { task: { kind: 'skill', id: 'writer', agent: 'injected' } });
    expect(store.chat.pendingExpert).toMatchObject({ plugin: 'writer', agent: 'installed-persona' });
    expect(store.chat.select).toHaveBeenCalledWith(null); expect(store.navigate).toHaveBeenCalledWith('new-task');
  });
  it('rejects removed plugins, disconnected examples and invalid indices before creating a task', async () => {
    const store = mockStore();
    ipc.mockResolvedValue([]);
    await expect(acceptStoreTask(store, { task: { kind: 'skill', id: 'removed' } })).rejects.toThrow();
    ipc.mockResolvedValue([{ id: 'docs', status: 'disconnected', examples: ['已验证示例'] }]);
    await expect(acceptStoreTask(store, { task: { kind: 'example', id: 'docs', example: 0 } })).rejects.toThrow();
    ipc.mockResolvedValue([{ id: 'docs', status: 'connected', examples: ['已验证示例'] }]);
    await expect(acceptStoreTask(store, { task: { kind: 'example', id: 'docs', example: -1 } })).rejects.toThrow();
    expect(store.chat.select).not.toHaveBeenCalled();
    await acceptStoreTask(store, { task: { kind: 'example', id: 'docs', example: 0 } });
    expect(store.creationDraft).toBe('已验证示例'); expect(store.navigate).toHaveBeenCalledWith('new-task');
  });
});
