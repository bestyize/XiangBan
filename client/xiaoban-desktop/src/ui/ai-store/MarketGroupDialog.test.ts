// @vitest-environment happy-dom
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '../../bridge.ts';
import type { Skill } from './catalog.ts';
import type { GroupMember } from './market.ts';
import MarketGroupDialog from './MarketGroupDialog.vue';

vi.mock('../../bridge.ts', () => ({ invoke: vi.fn() }));
vi.mock('naive-ui', async () => {
  const { defineComponent } = await import('vue');
  return {
    NModal: defineComponent({ props: ['show'], template: '<section v-if="show"><slot /></section>' }),
    NAlert: defineComponent({ template: '<div role="alert"><slot /></div>' }),
  };
});
const item: Skill = { slug: 'team', name: '开发专家团', namespace: 'acme', description: '', version: '1', variant: 'team' };
const member = (slug: string, state: 'installed' | 'pending' = 'installed'): GroupMember => ({ slug, name: slug, namespace: 'acme', sha256: 'hash', state, kind: 'experts', content: '', scripts: [], manifest: { version: '1', fileCount: 1, totalBytes: 1, files: [] } });
const saved = (slug: string): Skill => ({ slug, name: slug, description: '', namespace: 'acme', version: '1', market: true, removable: true, manifestSha256: 'hash' });
let members: GroupMember[], local: Skill[], wrapper: VueWrapper;
async function open() { wrapper = mount(MarketGroupDialog, { props: { item, kind: 'experts' } }); await flushPromises(); }
async function confirm() { await wrapper.get('.store-remove-confirm').trigger('click'); await flushPromises(); }
beforeEach(() => {
  members = [member('installed'), member('pending', 'pending')];
  local = [saved('installed'), saved('unrelated')];
  vi.mocked(invoke).mockReset().mockImplementation(async (command, args) => {
    if (command === 'ai_market_group_inspect') return { name: item.name, kind: 'experts', sha256: 'group-hash', members };
    if (command === 'ai_store_installed') return local;
    if (command === 'ai_store_uninstall') { local = local.filter(s => s.slug !== args?.slug); return; }
    throw new Error(`Unexpected ${command}`);
  });
});
afterEach(() => wrapper?.unmount());
describe('group package removal', () => {
  it('confirms the installed members and leaves pending and unrelated packages alone', async () => {
    await open(); await wrapper.get('.store-remove-entry').trigger('click');
    expect(wrapper.get('.group-remove-members').text()).toBe('installed');
    await wrapper.get('.store-removal .store-secondary').trigger('click');
    expect(invoke).not.toHaveBeenCalledWith('ai_store_uninstall', expect.anything());
    await wrapper.get('.store-remove-entry').trigger('click'); await confirm();
    expect(vi.mocked(invoke).mock.calls.filter(([c]) => c === 'ai_store_uninstall')).toEqual([['ai_store_uninstall', { slug: 'installed' }]]);
    expect(local.map(s => s.slug)).toEqual(['unrelated']);
    expect(wrapper.emitted('done')?.[0]).toEqual(['已移除「开发专家团」的 1 位专家']);
    expect(wrapper.emitted('changed')).toHaveLength(1);
  });
  it.each([{ namespace: 'another' }, { version: '2' }, { manifestSha256: 'different' }, { removable: false }])('refuses removal after the installed identity changes: %s', async patch => {
    local[0] = { ...local[0], ...patch };
    await open(); await wrapper.get('.store-remove-entry').trigger('click'); await confirm();
    expect(invoke).not.toHaveBeenCalledWith('ai_store_uninstall', expect.anything());
    expect(wrapper.text()).toContain('安装来源或版本已变化');
    expect(wrapper.emitted('done')).toBeUndefined();
  });
  it('reports partial failure and retries only the remaining installed members', async () => {
    members = [member('first'), member('second')]; local = [saved('first'), saved('second')];
    const original = vi.mocked(invoke).getMockImplementation()!;
    let failed = false;
    vi.mocked(invoke).mockImplementation(async (command, args) => {
      if (command === 'ai_store_uninstall' && args?.slug === 'second' && !failed) { failed = true; throw new Error('文件被占用'); }
      return original(command, args);
    });
    await open(); await wrapper.get('.store-remove-entry').trigger('click'); await confirm();
    expect(wrapper.text()).toContain('已移除 1 位专家；移除失败：');
    expect(wrapper.get('.group-remove-members').text()).toBe('second');
    expect(wrapper.emitted('done')).toBeUndefined();
    await confirm();
    expect(vi.mocked(invoke).mock.calls.filter(([c, args]) => c === 'ai_store_uninstall' && args?.slug === 'first')).toHaveLength(1);
    expect(local).toEqual([]);
    expect(wrapper.emitted('done')).toHaveLength(1);
  });
});
