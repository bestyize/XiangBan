import { createApp, ref, h } from 'vue';
import WidgetSurface from './WidgetSurface.vue';
import { desktop, invoke } from '../bridge.ts';
import type { WidgetPayload } from './model.ts';
import './window.css';
declare global { interface Window { __XIAOBAN_WIDGET__?: WidgetPayload } }
export async function mountWidget() {
  document.body.className='widget-scene';
  // The initialization snapshot predates subsequent resizes and can be stale on
  // a WebView reload. Ask the host for the current dimensions in native windows.
  const state=ref(desktop?await invoke<WidgetPayload>('get_desktop_widget'):window.__XIAOBAN_WIDGET__??await invoke<WidgetPayload>('get_desktop_widget'));
  const pixelRatio=ref(window.devicePixelRatio||1);
  const resized=()=>{pixelRatio.value=window.devicePixelRatio||1;};window.addEventListener('resize',resized);
  const app=createApp({render(){
    const ratio=state.value.displayScale?state.value.displayScale/pixelRatio.value:1;
    const scale=state.value.widget.scale*ratio,custom=state.value.widget.kind==='custom';
    // Render custom iframe windows at their final size; scale layout inside the
    // sandbox instead of magnifying the iframe's composited bitmap.
    return h('div',{class:'widget-frame',style:{transform:custom?'none':`scale(${scale})`,left:`${8*ratio}px`,top:`${8*ratio}px`}},[
      h(WidgetSurface,{widget:state.value.widget,editing:state.value.editing,payload:state.value,renderScale:custom?scale:1}),
    ]);
  }});
  const change=(event:Event)=>{state.value=(event as CustomEvent<WidgetPayload>).detail;};window.addEventListener('widget-config',change);
  app.mount('#app');window.addEventListener('pagehide',()=>{window.removeEventListener('widget-config',change);window.removeEventListener('resize',resized);app.unmount();},{once:true});
}

