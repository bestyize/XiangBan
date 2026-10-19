// @vitest-environment happy-dom
import {mount,flushPromises} from '@vue/test-utils';
import {describe,it,expect,vi,beforeEach} from 'vitest';
import WidgetsPage from './WidgetsPage.vue';
import type {ControlStore} from '../control-types.ts';
vi.mock('../../widgets/WidgetSurface.vue',()=>({default:{template:'<div class="surface" />'}}));
vi.stubGlobal('ResizeObserver',class{observe(){}disconnect(){}});
const store=()=>({desktop:false,notify:vi.fn()} as unknown as ControlStore);
describe('replacement widget library',()=>{
 beforeEach(()=>localStorage.clear());
 it('shows all eight reference cards without retired entry points',async()=>{const v=mount(WidgetsPage,{props:{store:store()}});await flushPromises();expect(v.findAll('.xb-widget-slot')).toHaveLength(8);for(const old of ['创作小组件','导入小组件','成长小白花','求签','外观样式'])expect(v.text()).not.toContain(old);await v.get('[data-testid="add-clock"]').trigger('click');await flushPromises();expect(JSON.parse(localStorage.getItem('xiaoban.widgets.v2')!).widgets[0]).toMatchObject({kind:'clock',theme:'ivory'});await v.get('.added-link').trigger('click');expect(v.findAll('.desktop-card')).toHaveLength(1);v.unmount();});
 it('consumes a missing native settings request',async()=>{const s=store(),v=mount(WidgetsPage,{props:{store:s,settingsRequest:{id:'missing'}}});await flushPromises();expect(v.emitted('settingsOpened')).toHaveLength(1);expect(s.notify).toHaveBeenCalledWith('这个组件已不在桌面上','info');v.unmount();});
});
