import {createApp,h,ref,watchEffect,nextTick} from 'vue';
import {invoke} from '../bridge.ts';
import {followHost,isDark} from '../ui/appearance.ts';
import {applyUiTokens} from '../ui/design-tokens.ts';
import WidgetSettingsMenu from './WidgetSettingsMenu.vue';
import type {DesktopWidget,WidgetMonitor,City} from './model.ts';
import './window.css';
export interface WidgetMenuPayload {widget:DesktopWidget;monitors:WidgetMonitor[];appearance:string;maxHeight:number}
declare global {interface Window {__XIAOBAN_WIDGET_MENU__?:WidgetMenuPayload}}
export async function mountWidgetSettings(){
 document.body.className='widget-menu-scene';
 const opening=ref(0);
 const state=ref(window.__XIAOBAN_WIDGET_MENU__??await invoke<WidgetMenuPayload>('get_widget_settings'));
 followHost(state.value.appearance);const stop=watchEffect(()=>applyUiTokens(isDark.value));
 const root=document.querySelector<HTMLElement>('#app')!;
 const accept=(data:WidgetMenuPayload)=>{state.value=data;followHost(data.appearance);};
 let sizing=false,queued=false,lastHeight=0,closed=false;
 async function fit(){
  if(sizing){queued=true;return;}const menu=root.querySelector<HTMLElement>('.widget-settings-menu');if(!menu||closed)return;
  const height=Math.ceil(menu.getBoundingClientRect().height+16);if(height===lastHeight)return;sizing=true;
  try{await invoke('size_widget_settings',{height});lastHeight=height;}catch{}finally{sizing=false;if(queued){queued=false;void fit();}}
 }
 const update=async(patch:Partial<DesktopWidget>)=>{const id=state.value.widget.id;const data=await invoke<WidgetMenuPayload>('update_widget_settings',{id,patch});if(id===state.value.widget.id&&data.widget.id===id)accept(data);};
 const close=()=>{void invoke('close_widget_settings').catch(()=>{});};
 const app=createApp({render:()=>h('div',{class:'widget-menu-frame',style:{'--widget-menu-max-height':`${state.value.maxHeight}px`}},[h(WidgetSettingsMenu,{key:state.value.widget.id+opening.value,widget:state.value.widget,monitors:state.value.monitors,update,remove:()=>invoke('remove_widget_settings',{id:state.value.widget.id}),searchCities:(query:string)=>invoke<City[]>('search_widget_cities',{query}),onClose:close})])});
 const change=(e:Event)=>{accept((e as CustomEvent<WidgetMenuPayload>).detail);void nextTick(fit);};
 const open=(e:Event)=>{opening.value++;lastHeight=0;change(e);};
 const appearance=(e:Event)=>followHost((e as CustomEvent).detail);
 const escape=(e:KeyboardEvent)=>{if(e.key==='Escape')close();};
 window.addEventListener('widget-menu-open',open);window.addEventListener('widget-menu-config',change);window.addEventListener('widget-menu-appearance',appearance);window.addEventListener('keydown',escape);
 app.mount(root);const observer=new ResizeObserver(()=>void fit());observer.observe(root.firstElementChild!);void fit();
 window.addEventListener('pagehide',()=>{closed=true;observer.disconnect();stop();window.removeEventListener('widget-menu-open',open);window.removeEventListener('widget-menu-config',change);window.removeEventListener('widget-menu-appearance',appearance);window.removeEventListener('keydown',escape);app.unmount();},{once:true});
}
