// @vitest-environment happy-dom
import {mount,flushPromises} from '@vue/test-utils';
import {reactive} from 'vue';
import {NSelect,NSwitch} from 'naive-ui';
import {beforeEach,describe,it,expect,vi} from 'vitest';
import {defaults,invoke} from '../../bridge.ts';
import type {ControlStore} from '../control-types.ts';
import CompanionPage from './CompanionPage.vue';
import { OFFICIAL_THEMES } from '../../pets/model.ts';
vi.mock('../../pets/PetActor.vue',()=>({default:{template:'<div class="actor"/>',props:['definition','size','expression']}}));
vi.mock('../../bridge.ts',async original=>({...await original<typeof import('../../bridge.ts')>(),desktop:false,invoke:vi.fn()}));
function setup(desktopUi=false){const settings=reactive({...defaults});const store={settings,desktop:desktopUi,notify:vi.fn(),startCreation:vi.fn().mockResolvedValue(undefined),setCompanion:vi.fn(async(enabled:boolean)=>{settings.companionEnabled=enabled;})} as unknown as ControlStore;return {store,view:mount(CompanionPage,{props:{store},attachTo:document.body})};}
async function tab(view:ReturnType<typeof mount>,name:string){await view.findAll('.n-tabs-tab').find(t=>t.text().startsWith(name))!.trigger('click');await flushPromises();}
async function add(view:ReturnType<typeof mount>,petId:string,layer:'desktop'|'topmost'='topmost'){await view.get(`[data-testid="pet-add-${petId}"]`).trigger('click');await flushPromises();const choice=document.body.querySelector<HTMLButtonElement>(`[data-testid="pet-placement-${layer}"]`);expect(choice).not.toBeNull();choice!.click();await flushPromises();}
beforeEach(()=>{localStorage.clear();vi.mocked(invoke).mockReset();});
describe('theme-based desktop pet library',()=>{
 it('only allows enabling deliberate, native companion greetings from the desktop pet tab',async()=>{
  vi.mocked(invoke).mockImplementation(async (command:string, args?:any)=>{
   if(command==='get_proactivity_settings')return {enabled:false,frequency:'natural'};
   if(command==='configure_proactivity')return args.settings;
   return undefined;
  });
  const {view}=setup(true);await flushPromises();
  expect(view.find('[data-testid="pet-proactivity-settings"]').exists()).toBe(false);
  await tab(view,'桌面伙伴');
  expect(view.get('[data-testid="pet-proactivity-settings"]').text()).toContain('默认关闭');
  const toggle=view.findAllComponents(NSwitch).find(c=>c.attributes('data-testid')==='pet-proactivity-switch')!;
  toggle.vm.$emit('update:value',true);await flushPromises();
  expect(invoke).toHaveBeenCalledWith('configure_proactivity',{settings:{enabled:true,frequency:'natural'}});
  const select=view.findAllComponents(NSelect).find(c=>c.attributes('data-testid')==='pet-proactivity-frequency')!;
  select.vm.$emit('update:value','quiet');await flushPromises();
  expect(invoke).toHaveBeenCalledWith('configure_proactivity',{settings:{enabled:true,frequency:'quiet'}});
  view.unmount();
 });
 it('shows official families and their members instead of a single active skin',async()=>{const {view}=setup();await flushPromises();expect(view.text()).toContain('官方主题');expect(view.text()).toContain('自定义主题');for(const name of ['花朵心语','多肉点点'])expect(view.text()).toContain(name);expect(view.findAll('.pet-theme-card')).toHaveLength(0);expect(view.text()).not.toContain('新宠物放在');expect(view.findAll('.pet-theme-detail').map(s=>s.find('h2').text())).toEqual(['花朵心语4 款宠物','多肉点点3 款宠物']);expect(view.find('[data-testid="pet-add-jade"]').exists()).toBe(true);expect(view.find('[data-testid="pet-add-daisy"]').exists()).toBe(true);expect(view.find('[data-testid="desktop-pets"]').exists()).toBe(false);expect(view.findAll('.n-tabs-tab').map(t=>t.text())).toEqual(['官方主题','自定义主题','桌面伙伴0']);view.unmount();});
 it('waits for placement selection and cancels without adding or showing pets',async()=>{const {view,store}=setup();await flushPromises();await view.get('[data-testid="pet-add-daisy"]').trigger('click');await flushPromises();expect(localStorage.getItem('xiaoban.pets.layout.v1')).toBeNull();expect(store.setCompanion).not.toHaveBeenCalled();const modal=document.body.querySelector('[data-testid="pet-placement-dialog"]')!;expect(modal.textContent).toContain('始终置顶');expect(modal.textContent).toContain('贴在桌面');(modal.querySelector('[data-testid="pet-placement-cancel"]') as HTMLButtonElement).click();await flushPromises();expect(localStorage.getItem('xiaoban.pets.layout.v1')).toBeNull();expect(view.find('[data-testid="pet-add-daisy"]').attributes('aria-expanded')).toBe('false');view.unmount();});
 it('adds duplicate pets and a second species as independent instances',async()=>{const {view}=setup();await flushPromises();await add(view,'daisy');await add(view,'daisy');await add(view,'jade','desktop');const layout=JSON.parse(localStorage.getItem('xiaoban.pets.layout.v1')!);expect(layout.instances).toHaveLength(3);expect(new Set(layout.instances.map((p:{id:string})=>p.id)).size).toBe(3);expect(layout.instances.map((p:{petId:string})=>p.petId)).toEqual(['daisy','daisy','jade']);expect(layout.instances.map((p:{layer:string})=>p.layer)).toEqual(['topmost','topmost','desktop']);expect(view.find('[data-testid="pet-add-daisy"]').exists()).toBe(true);await tab(view,'桌面伙伴');expect(view.findAll('.desktop-pet-row')).toHaveLength(3);expect(view.find('.pet-theme-detail').exists()).toBe(false);view.unmount();});
 it('changes the top plant of one existing pot without creating another instance',async()=>{const {view}=setup();await flushPromises();await add(view,'daisy');await tab(view,'桌面伙伴');await view.get('.pet-manage').trigger('click');await flushPromises();const select=view.findAllComponents(NSelect).find(c=>c.attributes('data-testid')==='pet-change-plant')!;expect(select.exists()).toBe(true);select.vm.$emit('update:value','succulent-dots/jade');await flushPromises();const layout=JSON.parse(localStorage.getItem('xiaoban.pets.layout.v1')!);expect(layout.instances).toHaveLength(1);expect(layout.instances[0]).toMatchObject({themeId:'succulent-dots',petId:'jade',size:210,expression:'idle'});view.unmount();});
 it('opens the requested pot in the desktop tab from both native entry paths',async()=>{const {view}=setup();await flushPromises();await add(view,'daisy');const {id}=JSON.parse(localStorage.getItem('xiaoban.pets.layout.v1')!).instances[0];await view.setProps({settingsRequest:{id}});await flushPromises();expect(view.find('[data-testid="desktop-pets"]').exists()).toBe(true);expect(view.findAllComponents(NSelect).some(c=>c.attributes('data-testid')==='pet-change-plant')).toBe(true);expect(view.emitted('settingsOpened')).toHaveLength(1);await tab(view,'官方主题');window.dispatchEvent(new CustomEvent('pets-manage',{detail:{id}}));await flushPromises();expect(view.find('[data-testid="desktop-pets"]').exists()).toBe(true);view.unmount();});
 it('offers the official library from an empty desktop tab',async()=>{const {view}=setup();await flushPromises();await tab(view,'桌面伙伴');expect(view.find('[data-testid="pet-custom-empty"]').exists()).toBe(false);await view.get('.desktop-pets-empty button').trigger('click');await flushPromises();expect(view.find('[data-testid="pet-add-daisy"]').exists()).toBe(true);view.unmount();});
 it('opens AI creation with a selected theme as context',async()=>{const {view,store}=setup();await flushPromises();await view.get('[data-testid="pet-create"]').trigger('click');await flushPromises();expect(store.startCreation).toHaveBeenCalledWith('pet');await view.get('[data-testid="pet-theme-flower-whispers"] .theme-actions button').trigger('click');await flushPromises();expect(store.creationDraft).toContain('flower-whispers');expect(store.creationDraft).toContain('现有主题');view.unmount();});
 it('shows explicit export and forum sharing for custom themes only, without a duplicate JSON export',async()=>{
  const custom={...OFFICIAL_THEMES[0],id:'my-pets',name:'我的宠物'};
  localStorage.setItem('xiaoban.pets.themes.v1',JSON.stringify([custom]));
  const {view}=setup(true);await flushPromises();
  expect(view.find('[data-testid="pet-theme-flower-whispers"] [data-testid="export-resource"]').exists()).toBe(false);
  expect(view.find('[data-testid="pet-theme-flower-whispers"] [data-testid="pet-export-reference"]').exists()).toBe(true);
  await tab(view,'自定义主题');
  const theme=view.get('[data-testid="pet-theme-my-pets"]');
  expect(theme.findAll('[data-testid="export-resource"]')).toHaveLength(1);
  expect(theme.find('[data-testid="pet-export-reference"]').exists()).toBe(false);
  expect(theme.findAll('[data-testid="share-resource"]')).toHaveLength(1);
  expect(theme.get('[data-testid="export-resource"]').text()).toContain('导出资源包');
  expect(theme.get('[data-testid="share-resource"]').text()).toContain('分享到论坛');
  expect(theme.get('[data-testid="pet-theme-export-note"]').text()).toContain('整套主题');
  view.unmount();
 });
 it('allows exporting a custom pet theme from the instance settings, never an official theme',async()=>{
  const custom={...OFFICIAL_THEMES[0],id:'my-pets',name:'我的宠物'};
  localStorage.setItem('xiaoban.pets.themes.v1',JSON.stringify([custom]));
  const {view}=setup(true);await flushPromises();await tab(view,'自定义主题');await add(view,'daisy');await tab(view,'桌面伙伴');
  await view.get('.pet-manage').trigger('click');await flushPromises();
  expect(document.body.querySelector('[data-testid="pet-instance-share"]')).not.toBeNull();
  expect(document.body.querySelector('[data-testid="pet-instance-share"] [data-testid="share-resource"]')).not.toBeNull();
  view.unmount();
 });
 it('keeps the instance list in its own tab and the custom empty state in custom themes',async()=>{const {view}=setup();await flushPromises();await add(view,'daisy');await tab(view,'自定义主题');expect(view.find('[data-testid="pet-custom-empty"]').exists()).toBe(true);expect(view.find('[data-testid="desktop-pets"]').exists()).toBe(false);await tab(view,'桌面伙伴');expect(view.find('[data-testid="pet-custom-empty"]').exists()).toBe(false);expect(view.findAll('.desktop-pet-row')).toHaveLength(1);view.unmount();});
});
