import { createApp, ref, h } from 'vue';
import WidgetSurface from './WidgetSurface.vue';
import { invoke } from '../bridge.ts';
import type { WidgetPayload } from './model.ts';
import './window.css';
declare global { interface Window { __XIAOBAN_WIDGET__?: WidgetPayload } }
export async function mountWidget() {
  document.body.className='widget-scene';
  const state=ref(window.__XIAOBAN_WIDGET__??await invoke<WidgetPayload>('get_desktop_widget'));
  const app=createApp({render(){return h('div',{class:'widget-frame',style:{transform:`scale(${state.value.widget.scale})`}},[h(WidgetSurface,{widget:state.value.widget,editing:state.value.editing,payload:state.value})]);}});
  const change=(event:Event)=>{state.value=(event as CustomEvent<WidgetPayload>).detail;};window.addEventListener('widget-config',change);
  app.mount('#app');window.addEventListener('pagehide',()=>{window.removeEventListener('widget-config',change);app.unmount();},{once:true});
}
