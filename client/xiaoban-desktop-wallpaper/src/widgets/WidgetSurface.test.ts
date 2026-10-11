// @vitest-environment happy-dom
import { mount } from '@vue/test-utils';
import { describe, it, expect, vi } from 'vitest';
import WidgetSurface from './WidgetSurface.vue';
import type { DesktopWidget } from './model.ts';
import { openRitual } from '../rituals/journal.ts';
import { invoke } from '../bridge.ts';
vi.mock('../bridge.ts',()=>({desktop:false,invoke:vi.fn().mockResolvedValue(undefined)}));
vi.mock('../rituals/FortuneVessel.vue',()=>({default:{template:'<div />'}}));
vi.mock('../rituals/journal.ts',async original=>({...await original<typeof import('../rituals/journal.ts')>(),openRitual:vi.fn().mockResolvedValue(undefined)}));
const one=(kind:DesktopWidget['kind']):DesktopWidget=>({id:'test',kind,theme:'paper',scale:1,x:.5,y:.5,enabled:true,city:null});
describe('desktop widget interactions',()=>{
  it('keeps a small stationary press as a click without moving the widget',async()=>{
    vi.useFakeTimers();vi.mocked(invoke).mockClear();
    const widget=one('fortune'),view=mount(WidgetSurface,{props:{widget,payload:{widget,editing:false,screenX:100,screenY:200}}});
    try{const button=view.get('.fortune-draw');await button.trigger('pointerdown',{button:0,pointerId:1,screenX:120,screenY:220});await button.trigger('pointerup',{button:0,pointerId:1,screenX:122,screenY:221});await button.trigger('click');
      expect(view.get('.fortune-draw').text()).toContain('正在途中');expect(invoke).not.toHaveBeenCalled();
    }finally{view.unmount();vi.useRealTimers();}
  });
  it('moves without edit mode, saves on release and suppresses the dragged button click',async()=>{
    vi.useFakeTimers();vi.mocked(invoke).mockClear();
    const widget=one('fortune'),view=mount(WidgetSurface,{props:{widget,payload:{widget,editing:false,screenX:100,screenY:200}}});
    try{await view.get('.fortune-draw').trigger('pointerdown',{button:0,pointerId:1,screenX:120,screenY:220});await view.get('article').trigger('pointermove',{pointerId:1,screenX:150,screenY:240});await view.get('article').trigger('pointerup',{pointerId:1,screenX:155,screenY:245});await view.get('.fortune-draw').trigger('click');
      expect(invoke).toHaveBeenLastCalledWith('move_desktop_widget',{x:135,y:225,commit:true});expect(view.get('.fortune-draw').text()).toBe('求一支好签');
      await vi.advanceTimersByTimeAsync(1);await view.get('.fortune-draw').trigger('click');expect(view.get('.fortune-draw').text()).toContain('正在途中');
    }finally{view.unmount();vi.useRealTimers();}
  });
  it('opens the same fortune that the widget revealed',async()=>{
    vi.useFakeTimers();const view=mount(WidgetSurface,{props:{widget:one('fortune'),preview:true}});
    try{await view.get('.fortune-draw').trigger('click');expect(view.get('.fortune-draw').attributes('disabled')).toBeDefined();await vi.advanceTimersByTimeAsync(1900);const number=Number(view.get('.mini-sign').text().match(/\d+/)?.[0])-1;
      await view.get('.fortune-draw').trigger('click');expect(openRitual).toHaveBeenLastCalledWith('fortune',number);
    }finally{view.unmount();vi.useRealTimers();}
  });
  it('locks drag-only editing without preventing calendar navigation',async()=>{
    const view=mount(WidgetSurface,{props:{widget:one('calendar'),preview:true}});const title=view.get('.month-name').text();await view.get('[aria-label="下个月"]').trigger('click');expect(view.get('.month-name').text()).not.toBe(title);await view.get('.month-name').trigger('click');expect(view.get('.month-name').text()).toBe(title);view.unmount();
  });
  it('shows configuration status instead of invented weather readings',()=>{const view=mount(WidgetSurface,{props:{widget:one('weather'),preview:true}});expect(view.text()).toContain('设置城市后显示天气');expect(view.get('.weather-main strong').text()).toBe('—');view.unmount();});
});
