import { createApp, ref, h } from 'vue';
import WidgetSurface from './WidgetSurface.vue';
import { invoke } from '../bridge.ts';
import type { WidgetPayload } from './model.ts';
import './window.css';
declare global { interface Window { __XIAOBAN_WIDGET__?: WidgetPayload } }
export async function mountWidget() {
  document.body.className='widget-scene';
  const state=ref(window.__XIAOBAN_WIDGET__??await invoke<WidgetPayload>('get_desktop_widget'));
  const pixelRatio=ref(window.devicePixelRatio||1);
  const resized=()=>{pixelRatio.value=window.devicePixelRatio||1;};window.addEventListener('resize',resized);
  const app=createApp({render(){const ratio=state.value.displayScale?state.value.displayScale/pixelRatio.value:1;return h('div',{class:'widget-frame',style:{transform:`scale(${state.value.widget.scale*ratio})`,left:`${8*ratio}px`,top:`${8*ratio}px`}},[h(WidgetSurface,{widget:state.value.widget,editing:state.value.editing,payload:state.value})]);}});
  const change=(event:Event)=>{state.value=(event as CustomEvent<WidgetPayload>).detail;};window.addEventListener('widget-config',change);
  app.mount('#app');window.addEventListener('pagehide',()=>{window.removeEventListener('widget-config',change);window.removeEventListener('resize',resized);app.unmount();},{once:true});
}

