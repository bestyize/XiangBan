// @vitest-environment happy-dom
import {mount,flushPromises} from '@vue/test-utils';
import {describe,it,expect,vi,beforeEach} from 'vitest';
import WidgetsPage from './WidgetsPage.vue';
import type {ControlStore} from '../control-types.ts';
vi.mock('../../widgets/WidgetSurface.vue',()=>({default:{template:'<div class="surface" />'}}));
vi.stubGlobal('ResizeObserver',class{observe(){}disconnect(){}});
const store=(desktop=true)=>({desktop,notify:vi.fn(),startCreation:vi.fn()} as unknown as ControlStore);
/** Adding opens a layer dialog (teleported), so the choice is found in the document body. */
async function addWith(v:ReturnType<typeof mount>,kind:string,layer:'desktop'|'topmost'){
 await v.get(`[data-testid="add-${kind}"]`).trigger('click');await flushPromises();
 const option=document.body.querySelector(`[data-testid="layer-${layer}-${kind}"]`) as HTMLElement|null;
 if(!option)throw Error(`no ${layer} choice`);option.click();await flushPromises();
}
describe('replacement widget library',()=>{
 beforeEach(()=>localStorage.clear());
 it('allows both built-in and custom additions after twelve widgets are already on the desktop',async()=>{
  localStorage.setItem('xiaoban.widgets.v2',JSON.stringify({version:2,widgets:Array.from({length:12},(_,i)=>({id:`clock${i}`,kind:'clock',theme:'ivory',scale:1,x:0,y:0,enabled:true,city:null}))}));
  localStorage.setItem('xiaoban.custom_widgets.v1',JSON.stringify([{schemaVersion:1,id:'kitty',name:'Kitty',html:'<p>Kitty</p>'}]));
  const v=mount(WidgetsPage,{props:{store:store()},attachTo:document.body});await flushPromises();
  expect(v.get('[data-testid="add-clock"]').attributes('disabled')).toBeUndefined();
  await addWith(v,'clock','desktop');
  expect(JSON.parse(localStorage.getItem('xiaoban.widgets.v2')!).widgets).toHaveLength(13);
  await v.get('[data-testid="widgets-scope-custom"]').trigger('click');await flushPromises();
  const button=v.get('[data-testid="add-custom-kitty"]');expect(button.attributes('disabled')).toBeUndefined();
  await button.trigger('click');await flushPromises();
  (document.body.querySelector('[data-testid="layer-topmost-kitty"]') as HTMLElement).click();await flushPromises();
  const widgets=JSON.parse(localStorage.getItem('xiaoban.widgets.v2')!).widgets;
  expect(widgets).toHaveLength(14);expect(widgets[13]).toMatchObject({kind:'custom',customId:'kitty',layer:'topmost'});
  v.unmount();
 });
 it('shows all eight gallery cards and import/create buttons in header',async()=>{
  const s=store();
  const v=mount(WidgetsPage,{props:{store:s},attachTo:document.body});
  await flushPromises();
  expect(v.findAll('.xb-widget-slot')).toHaveLength(8);
  for(const old of ['成长小白花','求签','外观样式','已添加']) expect(v.text()).not.toContain(old);
  expect(v.find('[data-testid="widgets-create"]').exists()).toBe(true);
  expect(v.text()).toContain('导入');
  expect(v.text()).toContain('让小伴创作');
  await addWith(v,'clock','desktop');
  expect(JSON.parse(localStorage.getItem('xiaoban.widgets.v2')!).widgets[0]).toMatchObject({kind:'clock',theme:'ivory',layer:'desktop'});
  v.unmount();
 });
 it('asks for the layer before adding, and keeps the widget off the desktop until one is picked',async()=>{
  const s=store(),v=mount(WidgetsPage,{props:{store:s},attachTo:document.body});
  await flushPromises();
  await v.get('[data-testid="add-clock"]').trigger('click');await flushPromises();
  const dialog=document.body.querySelector('[data-testid="widget-layer-dialog"]');
  expect(dialog).not.toBeNull();
  expect(dialog!.textContent).toContain('贴在桌面');
  expect(dialog!.textContent).toContain('始终置顶');
  expect(localStorage.getItem('xiaoban.widgets.v2')).toBeNull();
  (document.body.querySelector('[data-testid="widget-layer-dialog"] .layer-cancel') as HTMLElement).click();
  await flushPromises();
  expect(localStorage.getItem('xiaoban.widgets.v2')).toBeNull();
  expect(v.find('[data-testid="add-clock"]').attributes('aria-expanded')).toBe('false');
  v.unmount();
 });
 it('adds a widget to the always-on-top layer when that is chosen',async()=>{
  const v=mount(WidgetsPage,{props:{store:store()},attachTo:document.body});
  await flushPromises();
  await addWith(v,'weather','topmost');
  expect(JSON.parse(localStorage.getItem('xiaoban.widgets.v2')!).widgets[0]).toMatchObject({kind:'weather',layer:'topmost'});
  v.unmount();
 });
 it('switches to custom category and shows empty state',async()=>{
  const s=store(),v=mount(WidgetsPage,{props:{store:s},attachTo:document.body});
  await flushPromises();
  expect(v.findAll('.add-section .xb-seg button').map(b=>b.text())).toEqual(['官方','我的']);
  await v.get('[data-testid="widgets-scope-custom"]').trigger('click');
  await flushPromises();
  expect(v.find('[data-testid="custom-empty"]').exists()).toBe(true);
  expect(v.text()).toContain('还没有自定义组件');
  v.unmount();
 });
 it('consumes a missing native settings request',async()=>{
  const s=store(),v=mount(WidgetsPage,{props:{store:s,settingsRequest:{id:'missing'}}});
  await flushPromises();
  expect(v.emitted('settingsOpened')).toHaveLength(1);
  expect(s.notify).toHaveBeenCalledWith('这个组件已不在桌面上','info');
  v.unmount();
 });
});

describe('widgets on the desktop',()=>{
 beforeEach(()=>localStorage.clear());
 it('lists every widget on the desktop with its own layer switch and a more menu',async()=>{
  localStorage.setItem('xiaoban.widgets.v2',JSON.stringify({version:2,widgets:[{id:'clock1',kind:'clock',theme:'ivory',scale:1,x:0,y:0,enabled:true,city:null,layer:'desktop'}]}));
  const v=mount(WidgetsPage,{props:{store:store()},attachTo:document.body});await flushPromises();
  expect(v.find('[data-testid="desk-widget-clock1"]').exists()).toBe(true);
  expect(v.get('[data-testid="widgets-on-desktop"]').text()).toContain('1 个');
  await v.get('[data-testid="desk-layer-topmost-clock1"]').trigger('click');await flushPromises();
  expect(JSON.parse(localStorage.getItem('xiaoban.widgets.v2')!).widgets[0].layer).toBe('topmost');
  await v.get('[data-testid="desk-more-clock1"]').trigger('click');
  await v.get('[data-testid="desk-remove-clock1"]').trigger('click');await flushPromises();
  expect(JSON.parse(localStorage.getItem('xiaoban.widgets.v2')!).widgets).toHaveLength(0);
  v.unmount();
 });
});
