// @vitest-environment happy-dom
import {mount,flushPromises} from '@vue/test-utils';
import {nextTick} from 'vue';
import {describe,it,expect,vi} from 'vitest';
vi.mock('../bridge.ts',()=>({desktop:false,invoke:vi.fn().mockResolvedValue(undefined)}));
import WidgetSurface from './WidgetSurface.vue';
import {invoke} from '../bridge.ts';
import type {DesktopWidget} from './model.ts';
const one=(kind:DesktopWidget['kind']):DesktopWidget=>({id:'one',kind,theme:'ivory',scale:1,x:0,y:0,enabled:true,city:null});
describe('reference widget interaction',()=>{
 it('updates the clock at the next minute boundary',async()=>{
  vi.useFakeTimers();vi.setSystemTime(new Date('2026-10-20T12:34:15'));
  const view=mount(WidgetSurface,{props:{widget:one('clock'),preview:true}});
  try{expect(view.get('.clock-time').text()).toBe('12:34');vi.advanceTimersByTime(45030);await nextTick();expect(view.get('.clock-time').text()).toBe('12:35');}
  finally{view.unmount();vi.useRealTimers();}
 });
 it('keeps illustrative weather and stocks separate from actual readings',()=>{for(const kind of ['weather','stocks'] as const){const real=mount(WidgetSurface,{props:{widget:one(kind),preview:true}});expect(real.text()).toContain('—');expect(real.text()).not.toContain('309.600');real.unmount();}const demo=mount(WidgetSurface,{props:{widget:one('stocks'),preview:true,demo:true}});expect(demo.text()).toContain('309.600');expect(demo.findAll('.stock-periods button')).toHaveLength(4);demo.unmount();});
 it('saves todo completion and keeps the original content on failure',async()=>{const save=vi.fn().mockRejectedValue(Error('磁盘不可写'));const widget={...one('notes'),note:{title:'待办',mode:'todo' as const,text:'',items:[{id:'first',text:'写周报',done:false,time:'09:00'}]}};const view=mount(WidgetSurface,{props:{widget,save}});await view.get('.check-circle').trigger('click');await flushPromises();expect(save.mock.calls[0][0].note.items[0].done).toBe(true);expect(widget.note.items[0].done).toBe(false);expect(view.get('[role="alert"]').text()).toContain('磁盘不可写');view.unmount();});
 it('adds dated events with start, end and location',async()=>{const save=vi.fn().mockResolvedValue(undefined);const view=mount(WidgetSurface,{props:{widget:one('reminders'),save}});await view.get('.agenda-date').trigger('click');await view.get('[aria-label="事项名称"]').setValue('设计评审');await view.get('[aria-label="日程日期"]').setValue('2026-10-20');await view.get('[aria-label="开始时间"]').setValue('14:00');await view.get('[aria-label="结束时间"]').setValue('15:30');await view.get('[aria-label="日程地点"]').setValue('A302');await view.get('form').trigger('submit');expect(save.mock.calls[0][0].living.reminders[0]).toMatchObject({title:'设计评审',at:'2026-10-20T14:00',end:'15:30',location:'A302'});view.unmount();});
 it('keeps rapid woodfish taps while an earlier count is being saved',async()=>{const pending:Array<()=>void>=[];const save=vi.fn((_content:unknown)=>new Promise<void>(resolve=>pending.push(resolve)));const view=mount(WidgetSurface,{props:{widget:{...one('woodfish'),living:{count:108,symbol:'1.000001',reminders:[]}},save}});await view.get('.sound-toggle').trigger('click');pending.shift()!();await flushPromises();save.mockClear();await view.get('.fish-knock').trigger('click');await view.get('.fish-knock').trigger('click');await view.get('.fish-knock').trigger('click');expect(view.get('.fish-footer strong').text()).toContain('111');expect(save).toHaveBeenCalledTimes(1);pending.shift()!();await flushPromises();expect(save).toHaveBeenCalledTimes(2);expect(save.mock.calls[1][0]).toMatchObject({living:{count:111}});pending.shift()!();await flushPromises();view.unmount();});
 it('keeps taps made while the sound preference is being saved',async()=>{const pending:Array<()=>void>=[];const save=vi.fn((_c:unknown)=>new Promise<void>(resolve=>pending.push(resolve)));const widget={...one('woodfish'),living:{count:108,sound:true,symbol:'1.000001',reminders:[]}};const view=mount(WidgetSurface,{props:{widget,save}});await view.get('.sound-toggle').trigger('click');await view.get('.fish-knock').trigger('click');await view.get('.fish-knock').trigger('click');await view.setProps({widget:{...widget,living:{...widget.living,sound:false}}});pending.shift()!();await flushPromises();expect(view.get('.fish-footer strong').text()).toContain('110');expect(save.mock.calls[1][0]).toMatchObject({living:{count:110,sound:false}});pending.shift()!();await flushPromises();view.unmount();});
 it('restores the saved sound preference',()=>{const view=mount(WidgetSurface,{props:{widget:{...one('woodfish'),living:{count:4,sound:false,symbol:'1.000001',reminders:[]}}}});expect(view.find('[aria-label="打开木鱼声音"]').exists()).toBe(true);view.unmount();});
 it('navigates months and returns to the current month',async()=>{const v=mount(WidgetSurface,{props:{widget:one('calendar'),preview:true}});const title=v.get('.month-title').text();await v.get('[aria-label="下个月"]').trigger('click');expect(v.get('.month-title').text()).not.toBe(title);await v.get('.month-title').trigger('click');expect(v.get('.month-title').text()).toBe(title);v.unmount();});
});

describe('custom widget host drag',()=>{
 const custom=():DesktopWidget=>({...one('custom'),customId:'kitty',custom:{schemaVersion:1,id:'kitty',name:'Kitty',html:'<div>Kitty</div>',width:280,height:240,theme:'transparent'}});
 it('resizes the iframe viewport without reloading its document or resetting its interactions',async()=>{
  const widget=custom();
  const view=mount(WidgetSurface,{attachTo:document.body,props:{widget,renderScale:1.3}});
  const iframe=view.get('iframe').element as HTMLIFrameElement,srcdoc=iframe.srcdoc;
  const post=vi.spyOn(iframe.contentWindow!,'postMessage');
  await view.get('iframe').trigger('load');
  expect((view.element as HTMLElement).style.width).toBe('364px');
  expect(post).toHaveBeenLastCalledWith({type:'xiaoban-widget-layout',scale:1.3,width:280,height:240},'*');
  await view.setProps({renderScale:1.5});
  expect((view.element as HTMLElement).style.width).toBe('420px');
  expect(view.get('iframe').element).toBe(iframe);
  expect(iframe.srcdoc).toBe(srcdoc);
  expect(post).toHaveBeenLastCalledWith({type:'xiaoban-widget-layout',scale:1.5,width:280,height:240},'*');
  view.unmount();
 });
 it('accepts only its own iframe and saves the scaled grab position on release',async()=>{
  vi.mocked(invoke).mockClear();
  const widget=custom(),view=mount(WidgetSurface,{attachTo:document.body,props:{widget,payload:{widget,editing:false,screenX:0,screenY:0}}});
  const iframe=view.get('iframe').element as HTMLIFrameElement;
  vi.spyOn(iframe,'getBoundingClientRect').mockReturnValue({left:8,top:8,width:140,height:120} as DOMRect);
  const send=(phase:string,screenX=100,source:Window|null=iframe.contentWindow,x=.5)=>window.dispatchEvent(new MessageEvent('message',{source,data:{type:'xiaoban-widget-drag',phase,screenX,screenY:100,x,y:.5}}));
  send('start',100,window);send('move',120,window);send('end',120,window);
  send('start',100,iframe.contentWindow,2);send('move',120);send('end',120);
  await flushPromises();expect(invoke).not.toHaveBeenCalled();
  send('start');send('end');await flushPromises();expect(invoke).not.toHaveBeenCalled();
  send('start');send('move',120);await flushPromises();send('end',120);await flushPromises();
  expect(invoke).toHaveBeenLastCalledWith('move_desktop_widget',{grabX:78/innerWidth,grabY:68/innerHeight,commit:true});
  expect(iframe.getAttribute('sandbox')).toBe('allow-scripts');
  view.unmount();vi.mocked(invoke).mockClear();send('start');send('move',120);await flushPromises();expect(invoke).not.toHaveBeenCalled();
 });
 it('keeps preview and placement mode from accepting iframe commands',async()=>{
  for(const props of [{preview:true},{editing:true}]){
   vi.mocked(invoke).mockClear();const widget=custom();
   const view=mount(WidgetSurface,{attachTo:document.body,props:{widget,payload:{widget,editing:false,screenX:0,screenY:0},...props}});
   const source=(view.get('iframe').element as HTMLIFrameElement).contentWindow;
   for(const [phase,screenX] of [['start',100],['move',120],['end',120]])window.dispatchEvent(new MessageEvent('message',{source,data:{type:'xiaoban-widget-drag',phase,screenX,screenY:100,x:.5,y:.5}}));
   await flushPromises();expect(invoke).not.toHaveBeenCalled();view.unmount();
  }
 });
});
