// @vitest-environment happy-dom
import { mount, flushPromises } from '@vue/test-utils';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import ResourceShareActions from './ResourceShareActions.vue';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
vi.mock('../../bridge.ts',()=>({invoke:vi.fn()}));
const create = (id?:string, icons=false, kind:'wallpaper'|'pet'='wallpaper') => {
 const store={desktop:true,notify:vi.fn()} as unknown as ControlStore;
 const wrapper=mount(ResourceShareActions,{props:{store,kind,id,icons},global:{stubs:{NButton:{template:'<button v-bind="$attrs"><slot/><slot name="icon"/></button>'}}}});
 return {store,wrapper};
};
beforeEach(()=>vi.clearAllMocks());
describe('portable resource actions',()=>{
 it('exports the selected resource without publishing or opening the forum',async()=>{
  vi.mocked(invoke).mockResolvedValue(true);const {wrapper}=create('local-demo');
  await wrapper.get('[data-testid="export-resource"]').trigger('click');await flushPromises();
  expect(invoke).toHaveBeenCalledExactlyOnceWith('export_resource_pack',{kind:'wallpaper',id:'local-demo'});wrapper.unmount();
 });
 it('exports pet themes through the same ZIP resource route as custom widgets',async()=>{
  vi.mocked(invoke).mockResolvedValue(true);const {wrapper}=create('my-pets',false,'pet');
  await wrapper.get('[data-testid="export-resource"]').trigger('click');await flushPromises();
  expect(invoke).toHaveBeenCalledExactlyOnceWith('export_resource_pack',{kind:'pet',id:'my-pets'});
  wrapper.unmount();
 });
 it('opens the forum only after the share export succeeds',async()=>{
  vi.mocked(invoke).mockResolvedValue(true);const {wrapper}=create('local-demo');
  await wrapper.get('[data-testid="share-resource"]').trigger('click');await flushPromises();
  expect(invoke).toHaveBeenNthCalledWith(1,'export_resource_pack',{kind:'wallpaper',id:'local-demo'});
  expect(invoke).toHaveBeenNthCalledWith(2,'open_download_url',{url:'https://chat.xiangban.xyz'});wrapper.unmount();
 });
 it('cancelling the save dialog does not open the forum',async()=>{
  vi.mocked(invoke).mockResolvedValue(false);const {wrapper,store}=create('local-demo');
  await wrapper.get('[data-testid="share-resource"]').trigger('click');await flushPromises();
  expect(invoke).toHaveBeenCalledTimes(1);expect(store.notify).not.toHaveBeenCalled();wrapper.unmount();
 });
 it('refreshes the resource list after import without applying it',async()=>{
  const result={kind:'wallpaper',id:'imported',name:'星空'};vi.mocked(invoke).mockResolvedValue(result);const {wrapper}=create();
  await wrapper.get('[data-testid="import-resource"]').trigger('click');await flushPromises();
  expect(invoke).toHaveBeenCalledExactlyOnceWith('import_resource_pack');
  expect(wrapper.emitted('imported')).toEqual([[result]]);wrapper.unmount();
 });
 it('says where a pack of another kind went and leaves this page alone',async()=>{
  vi.mocked(invoke).mockResolvedValue({kind:'game',id:'star-catcher',name:'接星星'});const {wrapper,store}=create();
  await wrapper.get('[data-testid="import-resource"]').trigger('click');await flushPromises();
  expect(wrapper.emitted('imported')).toBeUndefined();
  expect(store.notify).toHaveBeenCalledWith('已导入「接星星」，在游戏空间页查看。','success');wrapper.unmount();
 });
 it('reports failed imports without emitting a success',async()=>{
  vi.mocked(invoke).mockRejectedValue('资源包版本暂不支持');const {wrapper,store}=create();
  await wrapper.get('[data-testid="import-resource"]').trigger('click');await flushPromises();
  expect(wrapper.emitted('imported')).toBeUndefined();expect(store.notify).toHaveBeenCalledWith('资源包版本暂不支持','error');wrapper.unmount();
 });
 it('keeps both verbs reachable when only the marks are drawn',async()=>{
  vi.mocked(invoke).mockResolvedValue(true);const {wrapper}=create('local-demo',true);
  expect(wrapper.get('.resource-share-actions').classes()).toContain('icons');
  // The wording goes away, so the accessible names have to carry it.
  expect(wrapper.get('[data-testid="export-resource"]').attributes('aria-label')).toBe('导出资源包');
  expect(wrapper.get('[data-testid="share-resource"]').attributes('aria-label')).toBe('分享到论坛');
  expect(wrapper.text()).not.toContain('导出资源包');
  await wrapper.get('[data-testid="share-resource"]').trigger('click');await flushPromises();
  expect(invoke).toHaveBeenNthCalledWith(2,'open_download_url',{url:'https://chat.xiangban.xyz'});wrapper.unmount();
 });
 it('still spells both verbs out when the marks are not asked for',()=>{
  const {wrapper}=create('local-demo');
  expect(wrapper.get('.resource-share-actions').classes()).not.toContain('icons');
  expect(wrapper.get('[data-testid="export-resource"]').text()).toContain('导出资源包');
  expect(wrapper.get('[data-testid="share-resource"]').text()).toContain('分享到论坛');
  wrapper.unmount();
 });
});
