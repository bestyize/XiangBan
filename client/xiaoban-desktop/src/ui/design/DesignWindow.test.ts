// @vitest-environment happy-dom
import {flushPromises,mount,type VueWrapper} from '@vue/test-utils';
import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {nextTick} from 'vue';
import DesignWindow from './DesignWindow.vue';
import {appearanceMode,accent} from '../appearance.ts';
import {tokensFor} from '../design-tokens.ts';
const mocks=vi.hoisted(()=>({invoke:vi.fn(),listen:vi.fn()}));
vi.mock('../../bridge.ts',()=>mocks);
let wrapper:VueWrapper|undefined;
const off=vi.fn();
const open=()=>{wrapper=mount(DesignWindow,{global:{stubs:{DesignWorkbench:true}}});return wrapper;};
beforeEach(()=>{vi.resetAllMocks();appearanceMode.value='light';accent.value='red';mocks.listen.mockResolvedValue(off);});
afterEach(()=>{wrapper?.unmount();wrapper=undefined;appearanceMode.value='system';accent.value='red';});
describe('standalone canvas appearance',()=>{
 it('loads host appearance and follows live changes including system mode',async()=>{
  mocks.invoke.mockResolvedValue({appearance:'dark',accent:'blue'});open();await flushPromises();
  expect(document.documentElement.dataset.theme).toBe('dark');
  expect(document.documentElement.style.getPropertyValue('--ui-surface')).toBe(tokensFor(true,'blue').surface);
  expect(document.documentElement.dataset.accent).toBe('blue');
  const changed=mocks.listen.mock.calls[0][1];
  changed({appearance:'light',accent:'green'});await nextTick();
  expect(document.documentElement.dataset.theme).toBe('light');
  expect(document.documentElement.style.getPropertyValue('--ui-btn')).toBe(tokensFor(false,'green').btn);
  changed({appearance:'system'});await nextTick();expect(appearanceMode.value).toBe('system');
  wrapper!.unmount();wrapper=undefined;expect(off).toHaveBeenCalledOnce();
 });
 it('keeps a newer settings event when the startup read resolves late',async()=>{
  let finish!:(value:unknown)=>void;
  mocks.invoke.mockImplementation(()=>new Promise(resolve=>{finish=resolve;}));open();await flushPromises();
  mocks.listen.mock.calls[0][1]({appearance:'dark',accent:'purple'});
  finish({appearance:'light',accent:'red'});await flushPromises();
  expect(document.documentElement.dataset.theme).toBe('dark');expect(accent.value).toBe('purple');
 });
 it('cleans up a listener that finishes registering after the window unmounts',async()=>{
  let finish!:(value:()=>void)=>void;
  mocks.listen.mockImplementation(()=>new Promise(resolve=>{finish=resolve;}));open();wrapper!.unmount();wrapper=undefined;
  finish(off);await flushPromises();expect(off).toHaveBeenCalledOnce();expect(mocks.invoke).not.toHaveBeenCalled();
 });
});
