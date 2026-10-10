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
 it('opens the preview full screen and Esc closes only the viewer',async()=>{
 const store=reactive({desktop:false,settings:JSON.parse(JSON.stringify(defaults)),library:{items:[{definition:{id:'actual-image',title:'真实图片',kind:'image',version:'1',resource:{url:'',sha256:'',bytes:0,mime:'image/png'}},installed:true,coverUrl:'/wallpapers/image.svg',source:'builtin'}],source:'',activeId:''},applyWallpaper:vi.fn(),notify:vi.fn()} as unknown as ControlStore);
 const wrapper=mount(WallpaperPage,{attachTo:document.body,props:{store},global:{stubs:{NButton:{template:'<button><slot /></button>'},NInput:stub,NSelect:stub,NSlider:stub,NSwitch:stub,NTabs:stub,NTabPane:stub,NModal:stub,NEmpty:stub,NAlert:stub,NPagination:stub,NCollapse:stub,NCollapseItem:stub,NColorPicker:stub,NSpin:stub}}});
 const viewer=()=>document.body.querySelector('[data-testid="fullscreen-viewer"]');
 await wrapper.get('[data-testid="wallpaper-card-actual-image"]').trigger('click');await flushPromises();
 expect(viewer()).toBeNull();
 await wrapper.get('[data-testid="open-fullscreen"]').trigger('click');await flushPromises();
 expect(viewer()).not.toBeNull();expect(viewer()!.getAttribute('aria-label')).toContain('真实图片');
 expect(document.activeElement?.getAttribute('data-testid')).toBe('close-fullscreen');
 window.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape'}));await flushPromises();
 expect(viewer()).toBeNull();expect(wrapper.find('[data-testid="wallpaper-inspector"]').exists()).toBe(true);
 await wrapper.get('[data-testid="inspector-preview"]').trigger('click');await flushPromises();
 expect(viewer()).not.toBeNull();
 (document.body.querySelector('[data-testid="close-fullscreen"]') as HTMLButtonElement).click();await flushPromises();
 expect(viewer()).toBeNull();expect(wrapper.find('[data-testid="wallpaper-inspector"]').exists()).toBe(true);
 wrapper.unmount();
 });
 it('keeps selecting and cancelling local previews away from native writes',async()=>{
 const apply=vi.fn();const store=reactive({desktop:false,settings:JSON.parse(JSON.stringify(defaults)),library:{items:[{definition:{id:'actual-image',title:'真实图片',kind:'image',version:'1',resource:{url:'',sha256:'',bytes:0,mime:'image/png'}},installed:true,coverUrl:'/wallpapers/image.svg',source:'builtin'}],source:'',activeId:''},applyWallpaper:apply,notify:vi.fn()} as unknown as ControlStore);
 const wrapper=mount(WallpaperPage,{attachTo:document.body,props:{store},global:{stubs:{NButton:{template:'<button><slot /></button>'},NInput:stub,NSelect:stub,NSlider:stub,NSwitch:stub,NTabs:stub,NTabPane:stub,NModal:stub,NEmpty:stub,NAlert:stub,NPagination:stub,NCollapse:stub,NCollapseItem:stub,NColorPicker:stub,NSpin:stub}}});
 await wrapper.get('[data-testid="wallpaper-card-actual-image"]').trigger('click');await flushPromises();expect(wrapper.find('[data-testid="wallpaper-inspector"]').exists()).toBe(true);expect(apply).not.toHaveBeenCalled();expect(wrapper.get('[data-testid="apply-wallpaper"]').element.closest('[data-testid="inspector-actions"]')).not.toBeNull();expect(wrapper.get('[data-testid="inspector-scroll"]').element.contains(wrapper.get('[data-testid="apply-wallpaper"]').element)).toBe(false);await wrapper.get('[data-testid="cancel-selection"]').trigger('click');expect(wrapper.find('[data-testid="wallpaper-inspector"]').exists()).toBe(false);expect(apply).not.toHaveBeenCalled();wrapper.unmount();
 });
 it('right-clicks select the target and apply through the material menu',async()=>{
 const apply=vi.fn().mockResolvedValue(undefined);const store=reactive({desktop:false,settings:JSON.parse(JSON.stringify(defaults)),library:{items:[{definition:{id:'context-image',title:'素材',kind:'image',version:'1',resource:{url:'',sha256:'',bytes:0,mime:'image/png'}},installed:true,coverUrl:'',source:'builtin'}],source:'',activeId:''},applyWallpaper:apply,notify:vi.fn()} as unknown as ControlStore);
 const wrapper=mount(WallpaperPage,{attachTo:document.body,props:{store},global:{stubs:{NButton:{template:'<button><slot /></button>'},NInput:stub,NSelect:stub,NSlider:stub,NSwitch:stub,NTabs:stub,NTabPane:stub,NModal:stub,NEmpty:stub,NAlert:stub,NPagination:stub,NCollapse:stub,NCollapseItem:stub,NColorPicker:stub,NSpin:stub}}});
 await wrapper.get('[data-testid="wallpaper-card-context-image"]').trigger('contextmenu',{clientX:50,clientY:60});await flushPromises();expect(wrapper.find('[data-testid="wallpaper-inspector"]').exists()).toBe(true);expect(apply).not.toHaveBeenCalled();const item=[...document.querySelectorAll('[role="menuitem"]')].find(node=>node.textContent==='应用到桌面') as HTMLButtonElement;expect(item).toBeTruthy();item.click();await flushPromises();expect(apply).toHaveBeenCalledWith('context-image',expect.any(Object),false);expect(document.querySelector('[data-testid="material-context-menu"]')).toBeNull();wrapper.unmount();
 });
});

describe('wallpaper tabs and subscription refresh',()=>{
 it('renders actual Naive tabs and refreshes the configured subscription remotely',async()=>{
 const store=reactive({desktop:false,settings:JSON.parse(JSON.stringify(defaults)),library:{items:[],source:'https://example.test/catalog.json',activeId:''},applyWallpaper:vi.fn(),notify:vi.fn(),refreshSettings:vi.fn().mockResolvedValue(undefined)} as unknown as ControlStore);
 const wrapper=mount(WallpaperPage,{attachTo:document.body,props:{store},global:{stubs:{NButton:{template:'<button><slot /></button>'},NInput:stub,NSelect:stub,NSlider:stub,NSwitch:stub,NModal:stub,NEmpty:stub,NAlert:stub,NPagination:stub,NCollapse:stub,NCollapseItem:stub,NColorPicker:stub,NSpin:stub}}});
 await flushPromises();expect(wrapper.findAll('.n-tabs-tab__label').map(node=>node.text())).toEqual(['在线资源','我的壁纸']);expect(wrapper.find('h1').exists()).toBe(false);expect(wrapper.find('.desktop-overview').exists()).toBe(false);store.desktop=true;vi.mocked(invoke).mockResolvedValue(store.library);await wrapper.findAll('button').find(button=>button.text()==='订阅与清单')!.trigger('click');await flushPromises();(document.querySelector('[data-testid="refresh-subscription"]') as HTMLButtonElement).click();await flushPromises();expect(invoke).toHaveBeenCalledWith('subscribe_wallpapers',{url:'https://example.test/catalog.json'});wrapper.unmount();
 });
});
