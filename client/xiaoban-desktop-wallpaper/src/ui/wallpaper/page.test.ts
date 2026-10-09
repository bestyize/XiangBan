// @vitest-environment happy-dom
import {mount,flushPromises} from '@vue/test-utils';
import {reactive} from 'vue';
import {describe,it,expect,vi} from 'vitest';
import WallpaperPage from '../pages/WallpaperPage.vue';
import {defaults,invoke} from '../../bridge.ts';
vi.mock('../../bridge.ts',async original=>({...await original<typeof import('../../bridge.ts')>(),invoke:vi.fn()}));
import type {ControlStore} from '../control-types.ts';
vi.mock('./MediaPreview.vue',()=>({default:{template:'<div />'}}));
const stub={props:['value','options'],template:'<div><slot /></div>'};
describe('wallpaper page interaction',()=>{
 it('keeps selecting and cancelling local previews away from native writes',async()=>{
 const apply=vi.fn();const store=reactive({desktop:false,settings:JSON.parse(JSON.stringify(defaults)),library:{items:[{definition:{id:'actual-image',title:'真实图片',kind:'image',version:'1',resource:{url:'',sha256:'',bytes:0,mime:'image/png'}},installed:true,coverUrl:'/wallpapers/image.svg',source:'builtin'}],source:'',activeId:''},applyWallpaper:apply,notify:vi.fn()} as unknown as ControlStore);
 const wrapper=mount(WallpaperPage,{attachTo:document.body,props:{store},global:{stubs:{NButton:{template:'<button><slot /></button>'},NInput:stub,NSelect:stub,NSlider:stub,NSwitch:stub,NTabs:stub,NTabPane:stub,NModal:stub,NEmpty:stub,NAlert:stub,NPagination:stub,NCollapse:stub,NCollapseItem:stub,NColorPicker:stub,NSpin:stub}}});
 await wrapper.get('[data-testid="wallpaper-card-actual-image"]').trigger('click');await flushPromises();expect(wrapper.find('[data-testid="wallpaper-inspector"]').exists()).toBe(true);expect(apply).not.toHaveBeenCalled();await wrapper.get('[data-testid="cancel-selection"]').trigger('click');expect(wrapper.find('[data-testid="wallpaper-inspector"]').exists()).toBe(false);expect(apply).not.toHaveBeenCalled();wrapper.unmount();
 });
});

describe('wallpaper tabs and subscription refresh',()=>{
 it('renders actual Naive tabs and refreshes the configured subscription remotely',async()=>{
 const store=reactive({desktop:false,settings:JSON.parse(JSON.stringify(defaults)),library:{items:[],source:'https://example.test/catalog.json',activeId:''},applyWallpaper:vi.fn(),notify:vi.fn(),refreshSettings:vi.fn().mockResolvedValue(undefined)} as unknown as ControlStore);
 const wrapper=mount(WallpaperPage,{attachTo:document.body,props:{store},global:{stubs:{NButton:{template:'<button><slot /></button>'},NInput:stub,NSelect:stub,NSlider:stub,NSwitch:stub,NModal:stub,NEmpty:stub,NAlert:stub,NPagination:stub,NCollapse:stub,NCollapseItem:stub,NColorPicker:stub,NSpin:stub}}});
 await flushPromises();expect(wrapper.findAll('.n-tabs-tab__label').map(node=>node.text())).toEqual(['在线资源','我的壁纸']);expect(wrapper.text()).toContain('系统桌面');store.settings.companionEnabled=false;await flushPromises();expect(wrapper.text()).toContain('小伴已隐藏');store.desktop=true;vi.mocked(invoke).mockResolvedValue(store.library);await wrapper.findAll('button').find(button=>button.text()==='订阅与清单')!.trigger('click');await flushPromises();(document.querySelector('[data-testid="refresh-subscription"]') as HTMLButtonElement).click();await flushPromises();expect(invoke).toHaveBeenCalledWith('subscribe_wallpapers',{url:'https://example.test/catalog.json'});wrapper.unmount();
 });
});
