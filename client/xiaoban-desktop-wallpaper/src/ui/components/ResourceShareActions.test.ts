// @vitest-environment happy-dom
import { mount, flushPromises } from '@vue/test-utils';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import ResourceShareActions from './ResourceShareActions.vue';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
vi.mock('../../bridge.ts',()=>({invoke:vi.fn()}));
const create = (id?:string) => {
 const store={desktop:true,notify:vi.fn()} as unknown as ControlStore;
 const wrapper=mount(ResourceShareActions,{props:{store,kind:'wallpaper',id},global:{stubs:{NButton:{template:'<button><slot/></button>'}}}});
 return {store,wrapper};
};
beforeEach(()=>vi.clearAllMocks());
describe('portable resource actions',()=>{
 it('exports the selected resource without publishing or opening the forum',async()=>{
  vi.mocked(invoke).mockResolvedValue(true);const {wrapper}=create('local-demo');
  await wrapper.get('[data-testid="export-resource"]').trigger('click');await flushPromises();
  expect(invoke).toHaveBeenCalledExactlyOnceWith('export_resource_pack',{kind:'wallpaper',id:'local-demo'});wrapper.unmount();
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
  expect(invoke).toHaveBeenCalledExactlyOnceWith('import_resource_pack',{kind:'wallpaper'});
  expect(wrapper.emitted('imported')).toEqual([[result]]);wrapper.unmount();
 });
 it('reports failed imports without emitting a success',async()=>{
  vi.mocked(invoke).mockRejectedValue('资源包版本暂不支持');const {wrapper,store}=create();
  await wrapper.get('[data-testid="import-resource"]').trigger('click');await flushPromises();
  expect(wrapper.emitted('imported')).toBeUndefined();expect(store.notify).toHaveBeenCalledWith('资源包版本暂不支持','error');wrapper.unmount();
 });
});
