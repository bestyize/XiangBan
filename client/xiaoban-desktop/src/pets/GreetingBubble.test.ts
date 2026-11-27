// @vitest-environment happy-dom
import {mount,flushPromises} from '@vue/test-utils';
import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import GreetingBubble from './GreetingBubble.vue';

const host=vi.hoisted(()=>({invoke:vi.fn()}));
vi.mock('../bridge.ts',()=>({invoke:host.invoke}));

describe('opted-in desktop pet greeting',()=>{
 beforeEach(()=>{
  document.body.innerHTML='';
  host.invoke.mockReset();
  host.invoke.mockImplementation(async (command:string)=>{
   if(command==='get_companion_greeting')return {id:7,petName:'小花',text:'有时候什么都不做，也挺好的。'};
  });
 });
 afterEach(()=>{document.body.replaceChildren();document.body.className='';});
 it('renders the selected companion and sends the action id with a manual response',async()=>{
  const wrapper=mount(GreetingBubble,{attachTo:document.body});
  await flushPromises();
  expect(wrapper.text()).toContain('小花');
  expect(wrapper.text()).toContain('有时候什么都不做');
  const buttons=wrapper.findAll('button');
  expect(buttons.map(b=>b.text())).toContain('摸摸头');
  await buttons.find(b=>b.text()==='摸摸头')!.trigger('click');
  await flushPromises();
  expect(host.invoke).toHaveBeenCalledWith('respond_companion_greeting',{id:7,outcome:'poke'});
  wrapper.unmount();
 });
 it('sends an explicit snooze rather than treating it as an ignored bubble',async()=>{
  const wrapper=mount(GreetingBubble,{attachTo:document.body});
  await flushPromises();
  await wrapper.findAll('button').find(b=>b.text()==='暂不打扰')!.trigger('click');
  await flushPromises();
  expect(host.invoke).toHaveBeenCalledWith('respond_companion_greeting',{id:7,outcome:'snoozed'});
  wrapper.unmount();
 });
 it('reuses the mounted bubble for a later greeting and cancels the previous timeout',async()=>{
  vi.useFakeTimers();
  let complete!: (value:unknown)=>void;
  host.invoke.mockImplementationOnce(()=>new Promise(resolve=>{complete=resolve;}));
  const wrapper=mount(GreetingBubble);
  const change=(detail:unknown)=>window.dispatchEvent(new CustomEvent('companion-greeting-changed',{detail}));
  try {
   change({id:8,petName:'溟月',text:'一起看书吗？'});
   complete({id:7,petName:'小花',text:'先前的消息'});
   await flushPromises();
   expect(wrapper.text()).toContain('一起看书吗？');
   await vi.advanceTimersByTimeAsync(5000);
   change({id:9,petName:'溟月',text:'又见面了。'});
   await vi.advanceTimersByTimeAsync(3100);
   expect(wrapper.text()).toContain('又见面了。');
   expect(host.invoke).not.toHaveBeenCalledWith('respond_companion_greeting',expect.anything());
   change(null);
   await vi.advanceTimersByTimeAsync(8000);
   expect(wrapper.text()).toBe('');
   expect(host.invoke).not.toHaveBeenCalledWith('respond_companion_greeting',expect.anything());
  } finally {wrapper.unmount();vi.useRealTimers();}
 });
 it('applies host theme tokens, tail direction, and pauses auto-dismiss while hovered',async()=>{
  vi.useFakeTimers();
  host.invoke.mockImplementation(async (command:string)=>{
   if(command==='get_companion_greeting')return {id:10,petName:'向日葵',text:'嘿，陪我待会儿吗？',appearance:'dark',accent:'green',tail:'right'};
  });
  const wrapper=mount(GreetingBubble,{attachTo:document.body});
  try {
   await flushPromises();
   expect(document.documentElement.dataset.theme).toBe('dark');
   expect(document.documentElement.dataset.accent).toBe('green');
   const card=wrapper.get('.greeting-card');
   expect(card.attributes('data-tail')).toBe('right');
   await card.trigger('pointerenter');
   await vi.advanceTimersByTimeAsync(15_000);
   expect(host.invoke).not.toHaveBeenCalledWith('respond_companion_greeting',expect.anything());
   await card.trigger('pointerleave');
   await vi.advanceTimersByTimeAsync(8_000);
   await flushPromises();
   expect(host.invoke).toHaveBeenCalledWith('respond_companion_greeting',{id:10,outcome:'ignored'});
  } finally {wrapper.unmount();vi.useRealTimers();}
 });
});
