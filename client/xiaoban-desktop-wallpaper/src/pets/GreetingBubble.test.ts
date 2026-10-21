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
});
