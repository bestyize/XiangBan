// @vitest-environment happy-dom
import {mount} from '@vue/test-utils';
import {nextTick} from 'vue';
import {expect,it,vi} from 'vitest';
import WidgetPreview from './WidgetPreview.vue';
import {baseSize,type DesktopWidget} from '../../widgets/model.ts';
vi.mock('../../widgets/WidgetSurface.vue',()=>({default:{template:'<div/>'}}));
it('refits the preview when its card grows from five-column width to three-column width',async()=>{
 let resize:ResizeObserverCallback=()=>{};
 const disconnect=vi.fn();
 vi.stubGlobal('ResizeObserver',class {constructor(callback:ResizeObserverCallback){resize=callback;}observe(){}disconnect=disconnect;});
 const widget:DesktopWidget={id:'clock',kind:'clock',theme:'ivory',scale:.5,x:0,y:0,enabled:true,city:null};
 const view=mount(WidgetPreview,{props:{widget,demo:true}});
 const size=baseSize('clock');
 for(const [width,height] of [[120,100],[240,200]]){
  resize([{contentRect:{width,height}} as ResizeObserverEntry],{} as ResizeObserver);await nextTick();
  const scale=Math.min((width-28)/size.width,(height-28)/size.height);
  expect(view.get('.preview-inner').attributes('style')).toContain(`scale(${scale})`);
 }
 view.unmount();expect(disconnect).toHaveBeenCalledOnce();vi.unstubAllGlobals();
});
