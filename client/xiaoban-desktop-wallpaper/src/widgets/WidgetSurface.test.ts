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
  it('opens its own settings with a gear button without starting a drag',async()=>{
    vi.mocked(invoke).mockClear();
    const widget=one('calendar'),view=mount(WidgetSurface,{props:{widget,payload:{widget,editing:false,screenX:100,screenY:200}}});
    try{
      const button=view.get('[aria-label="设置这个组件"]');
      expect(button.find('.lucide-settings').exists()).toBe(true);
      expect(button.attributes('aria-haspopup')).toBe('dialog');
      await button.trigger('pointerdown',{button:0,pointerId:1,screenX:120,screenY:220});
      await view.get('article').trigger('pointermove',{pointerId:1,screenX:150,screenY:240});
      await view.get('article').trigger('pointerup',{pointerId:1,screenX:150,screenY:240});
      await button.trigger('click');
      expect(invoke).toHaveBeenCalledExactlyOnceWith('show_widget_settings');
    }finally{view.unmount();}
  });
  it('keeps a small stationary press as a click without moving the widget',async()=>{
    vi.useFakeTimers();vi.mocked(invoke).mockClear();
    const widget=one('fortune'),view=mount(WidgetSurface,{props:{widget,payload:{widget,editing:false,screenX:100,screenY:200}}});
    try{const button=view.get('.fortune-draw');await button.trigger('pointerdown',{button:0,pointerId:1,screenX:120,screenY:220});await button.trigger('pointerup',{button:0,pointerId:1,screenX:122,screenY:221});await button.trigger('click');
      expect(view.get('.fortune-draw').text()).toContain('正在途中');expect(invoke).not.toHaveBeenCalled();
    }finally{view.unmount();vi.useRealTimers();}
  });
  it('drags from the heading without edit mode and saves on release',async()=>{
    vi.useFakeTimers();vi.mocked(invoke).mockClear();
    const widget=one('fortune'),view=mount(WidgetSurface,{props:{widget,payload:{widget,editing:false,screenX:100,screenY:200}}});
    try{await view.get('.widget-heading span').trigger('pointerdown',{button:0,pointerId:1,screenX:120,screenY:220});await view.get('article').trigger('pointermove',{pointerId:1,screenX:150,screenY:240});await view.get('article').trigger('pointerup',{pointerId:1,screenX:155,screenY:245});await view.get('.fortune-draw').trigger('click');
      expect(invoke).toHaveBeenLastCalledWith('move_desktop_widget',{grabX:0,grabY:0,commit:true});expect(view.get('.fortune-draw').text()).toBe('求一支好签');
      await vi.advanceTimersByTimeAsync(1);await view.get('.fortune-draw').trigger('click');expect(view.get('.fortune-draw').text()).toContain('正在途中');
    }finally{view.unmount();vi.useRealTimers();}
  });
  it('keeps content presses and button gestures from moving the widget',async()=>{
    vi.useFakeTimers();vi.mocked(invoke).mockClear();
    const widget=one('fortune'),view=mount(WidgetSurface,{props:{widget,payload:{widget,editing:false,screenX:100,screenY:200}}});
    try{
      for(const selector of ['article','.mini-vessel','.fortune-draw']){
        await view.get(selector).trigger('pointerdown',{button:0,pointerId:1,screenX:120,screenY:220});
        await view.get('article').trigger('pointermove',{pointerId:1,screenX:150,screenY:240});
        await view.get('article').trigger('pointerup',{pointerId:1,screenX:150,screenY:240});
      }
      expect(invoke).not.toHaveBeenCalled();
      await view.get('.fortune-draw').trigger('click');expect(view.text()).toContain('好签正在途中');
    }finally{view.unmount();vi.useRealTimers();}
  });
  it('keeps a full-width heading drag area on custom widgets',async()=>{
    vi.mocked(invoke).mockClear();
    const widget={...one('custom'),customId:'demo',custom:{id:'demo',name:'自创组件',html:'<button>内容按钮</button>'}};
    const view=mount(WidgetSurface,{props:{widget,payload:{widget,editing:false,screenX:100,screenY:200}}});
    try{
      await view.get('.widget-heading').trigger('pointerdown',{button:0,pointerId:1,screenX:120,screenY:220});
      await view.get('article').trigger('pointerup',{pointerId:1,screenX:150,screenY:240});
      expect(invoke).toHaveBeenLastCalledWith('move_desktop_widget',{grabX:0,grabY:0,commit:true});
    }finally{view.unmount();}
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
  it('tells the preview that live weather only appears on the desktop, and has no hidden drag handle while editing',()=>{
    const city={name:'北京',country:'中国',admin1:'北京',latitude:39.9,longitude:116.4};
    const view=mount(WidgetSurface,{props:{widget:{...one('weather'),city},preview:true,editing:true}});
    expect(view.text()).toContain('实时天气显示在桌面上');expect(view.find('.drag-handle').exists()).toBe(false);expect(view.find('.widget-settings').exists()).toBe(false);view.unmount();
  });
  it('lets note inputs select text without moving the desktop window, and saves through the own-content command',async()=>{
    vi.mocked(invoke).mockClear();const widget={...one('notes'),note:{title:'随手记',mode:'note' as const,text:'',items:[]}};
    const view=mount(WidgetSurface,{props:{widget,payload:{widget,editing:false,screenX:100,screenY:200}}});
    await view.get('textarea').trigger('pointerdown',{button:0,pointerId:1,screenX:120,screenY:220});
    await view.get('article').trigger('pointermove',{pointerId:1,screenX:150,screenY:240});await view.get('article').trigger('pointerup',{pointerId:1,screenX:150,screenY:240});expect(invoke).not.toHaveBeenCalled();
    await view.get('textarea').setValue('明天开会');await view.get('.notes-content footer button').trigger('click');expect(invoke).toHaveBeenCalledWith('save_desktop_widget_content',{content:{note:{...widget.note,text:'明天开会'}}});view.unmount();
  });

  it('waters the white flower through the widget scene and saves only its own growth record',async()=>{
    vi.mocked(invoke).mockClear();
    const widget={...one('flower'),flower:{water:6,since:'2026-10-01'}};
    const view=mount(WidgetSurface,{props:{widget,payload:{widget,editing:false,screenX:100,screenY:200}}});
    expect(view.text()).toContain('小白花初开');
    await view.get('.flower-water').trigger('click');
    expect(invoke).toHaveBeenLastCalledWith('save_desktop_widget_content',{content:{flower:{water:7,since:'2026-10-01'}}});
    expect(view.text()).toContain('含苞');
    view.unmount();
  });
  it('uses a viewport grab fraction rather than browser global screen coordinates for cross-screen dragging',async()=>{
    vi.mocked(invoke).mockClear();vi.stubGlobal('innerWidth',400);vi.stubGlobal('innerHeight',400);
    const widget=one('calendar'),view=mount(WidgetSurface,{props:{widget,payload:{widget,editing:false,screenX:-1920,screenY:-900}}});
    try{await view.get('.widget-heading').trigger('pointerdown',{button:0,pointerId:1,clientX:100,clientY:40,screenX:-1820,screenY:-860});await view.get('article').trigger('pointermove',{pointerId:1,screenX:2200,screenY:100});await view.get('article').trigger('pointerup',{pointerId:1,screenX:2200,screenY:100});expect(invoke).toHaveBeenLastCalledWith('move_desktop_widget',{grabX:.25,grabY:.1,commit:true});}finally{view.unmount();vi.unstubAllGlobals();}
  });

});

