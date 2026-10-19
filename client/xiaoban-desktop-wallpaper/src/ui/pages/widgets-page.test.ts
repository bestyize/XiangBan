// @vitest-environment happy-dom
import {mount,flushPromises} from '@vue/test-utils';
import {describe,it,expect,vi,beforeEach} from 'vitest';
import WidgetsPage from './WidgetsPage.vue';
import type {ControlStore} from '../control-types.ts';
vi.mock('../../widgets/WidgetSurface.vue',()=>({default:{template:'<div class="surface" />'}}));
vi.stubGlobal('ResizeObserver',class{observe(){}disconnect(){}});
const store=()=>({desktop:false,notify:vi.fn()} as unknown as ControlStore);
/** Adding opens a layer dialog (teleported), so the choice is found in the document body. */
async function addWith(v:ReturnType<typeof mount>,kind:string,layer:'desktop'|'topmost'){
 await v.get(`[data-testid="add-${kind}"]`).trigger('click');await flushPromises();
 const option=document.body.querySelector(`[data-testid="layer-${layer}-${kind}"]`) as HTMLElement|null;
 if(!option)throw Error(`no ${layer} choice`);option.click();await flushPromises();
}
describe('replacement widget library',()=>{
 beforeEach(()=>localStorage.clear());
 it('shows all eight reference cards without retired entry points',async()=>{const v=mount(WidgetsPage,{props:{store:store()},attachTo:document.body});await flushPromises();expect(v.findAll('.xb-widget-slot')).toHaveLength(8);for(const old of ['创作小组件','导入小组件','成长小白花','求签','外观样式'])expect(v.text()).not.toContain(old);await addWith(v,'clock','desktop');expect(JSON.parse(localStorage.getItem('xiaoban.widgets.v2')!).widgets[0]).toMatchObject({kind:'clock',theme:'ivory',layer:'desktop'});await v.get('.added-link').trigger('click');expect(v.findAll('.desktop-card')).toHaveLength(1);v.unmount();});
 it('asks for the layer before adding, and keeps the widget off the desktop until one is picked',async()=>{const s=store(),v=mount(WidgetsPage,{props:{store:s},attachTo:document.body});await flushPromises();await v.get('[data-testid="add-clock"]').trigger('click');await flushPromises();const dialog=document.body.querySelector('[data-testid="widget-layer-dialog"]');expect(dialog).not.toBeNull();expect(dialog!.textContent).toContain('贴在桌面');expect(dialog!.textContent).toContain('始终置顶');expect(localStorage.getItem('xiaoban.widgets.v2')).toBeNull();(document.body.querySelector('[data-testid="widget-layer-dialog"] .layer-cancel') as HTMLElement).click();await flushPromises();expect(localStorage.getItem('xiaoban.widgets.v2')).toBeNull();expect(v.find('[data-testid="add-clock"]').attributes('aria-expanded')).toBe('false');v.unmount();});
 it('adds a widget to the always-on-top layer when that is chosen',async()=>{const v=mount(WidgetsPage,{props:{store:store()},attachTo:document.body});await flushPromises();await addWith(v,'weather','topmost');expect(JSON.parse(localStorage.getItem('xiaoban.widgets.v2')!).widgets[0]).toMatchObject({kind:'weather',layer:'topmost'});v.unmount();});
 it('consumes a missing native settings request',async()=>{const s=store(),v=mount(WidgetsPage,{props:{store:s,settingsRequest:{id:'missing'}}});await flushPromises();expect(v.emitted('settingsOpened')).toHaveLength(1);expect(s.notify).toHaveBeenCalledWith('这个组件已不在桌面上','info');v.unmount();});
});
