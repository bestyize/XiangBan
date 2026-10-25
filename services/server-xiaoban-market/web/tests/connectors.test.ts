import { mount, flushPromises } from '@vue/test-utils';
import { afterEach, expect, it, vi } from 'vitest';
import ItemCard from '../src/components/ItemCard.vue';
import DetailModal from '../src/components/DetailModal.vue';
import LibraryView from '../src/views/admin/LibraryView.vue';
import { admin, market } from '../src/lib/api';
import type { ItemDetail } from '../src/lib/types';

const connector = { displayName: { zh: '连接器', en: 'Connector' }, displayDescription: { zh: '', en: '' }, examples: { zh: ['查看我的仓库'], en: [] }, authTypes: ['token'] as const, tokenFields: [], transports: ['sse'], skills: ['helper'], skillCount: 1, disabledToolCount: 2 };
const item = { id: 'a', name: '连接器', slug: 'connector', namespace: 'ns', kind: 'connector', variant: 'single', summary: '简介', connector, tags: [], heat: 0, version: '1.0.0', downloads: 0, favorites: 0, featured: false, origin: 'upload', source: 'community', category: null, iconUrl: null, memberCount: null, updatedAt: '' };
const stubs = { RouterLink: { template: '<a><slot /></a>' } };
afterEach(() => vi.restoreAllMocks());
it('shows auth, an example and bundled skill count on connector cards', () => {
  const w = mount(ItemCard, { props: { item: item as unknown as ItemDetail['item'], to: '/' }, global: { stubs } });
  expect(w.text()).toContain('令牌认证');
  expect(w.text()).toContain('查看我的仓库');
  expect(w.text()).toContain('1 个随包技能');
  w.unmount();
});
it('shows connector details and bundled skills without credential values', async () => {
  vi.spyOn(market, 'detail').mockResolvedValue({ item: { ...item, description: '', prompts: ['查看我的仓库'], metadata: {}, originUrl: null, license: null, publishedAt: null }, versions: [], files: [], members: [] } as unknown as ItemDetail);
  const w = mount(DetailModal, { props: { kind: 'connectors', ns: 'ns', slug: 'connector' }, global: { stubs } });
  await flushPromises();
  for (const text of ['令牌认证', 'sse', '隐藏 2 个工具', 'helper', '查看我的仓库']) expect(w.text()).toContain(text);
  w.unmount();
});

it('allows uploading a connector plugin from the library', async () => {
  vi.spyOn(admin, 'items').mockResolvedValue({ results: [], total: 0, page: 1, pageSize: 50 });
  vi.spyOn(admin, 'categories').mockResolvedValue({ results: [] });
  vi.spyOn(admin, 'item').mockResolvedValue({ item: { ...item, status: 'draft' }, versions: [], members: [] } as unknown as Awaited<ReturnType<typeof admin.item>>);
  const upload = vi.spyOn(admin, 'importPlugin').mockResolvedValue({ itemId: 'a', version: '1.0.0', created: true, issues: [] });
  vi.spyOn(window, 'confirm').mockReturnValue(true);
  const w = mount(LibraryView);
  await flushPromises();
  await w.findAll('button').find(b => b.text() === '连接器')!.trigger('click');
  await flushPromises();
  expect(w.text()).toContain('上传插件包');
  const input = w.get('[data-testid="plugin-input"]');
  const file = new File(['zip'], 'connector.zip', { type: 'application/zip' });
  Object.defineProperty(input.element, 'files', { value: [file] });
  await input.trigger('change');
  await flushPromises();
  expect(upload).toHaveBeenCalledOnce();
  expect(upload.mock.calls[0][0].get('file')).toBe(file);
  w.unmount();
});


it('shows official provenance and unknown license and combines both library filters', async () => {
  const official = { ...item, origin: 'workbuddy-official', license: 'unknown', status: 'draft', attribution: { author: 'Original author' } };
  const query = vi.spyOn(admin, 'items').mockResolvedValue({ results: [official], total: 1, page: 1, pageSize: 50 } as unknown as Awaited<ReturnType<typeof admin.items>>);
  vi.spyOn(admin, 'categories').mockResolvedValue({ results: [] });
  vi.spyOn(admin, 'item').mockResolvedValue({ item: official, versions: [], members: [] } as unknown as Awaited<ReturnType<typeof admin.item>>);
  const w = mount(LibraryView);
  await flushPromises();
  expect(w.get('table').text()).toContain('WorkBuddy 官方市场');
  expect(w.get('table').text()).toContain('许可未知');
  await w.get('select[aria-label="来源"]').setValue('workbuddy-official');
  await w.get('select[aria-label="许可"]').setValue('unknown');
  await flushPromises();
  expect(query).toHaveBeenLastCalledWith(expect.objectContaining({ origin: 'workbuddy-official', license: 'unknown' }));
  await w.get('tbody tr').trigger('click');
  await flushPromises();
  expect(w.get('[data-testid="attribution"]').text()).toContain('Original author');
  expect(w.text()).toContain('逐个确认来源和许可后上架');
  expect(w.findAll('button').some(b => /全部发布|全部上架|一键.*发布/.test(b.text()))).toBe(false);
  w.unmount();
});
