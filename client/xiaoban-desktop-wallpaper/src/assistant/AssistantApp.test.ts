// @vitest-environment happy-dom
import { mount,flushPromises } from '@vue/test-utils';
import { reactive, nextTick } from 'vue';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import AssistantApp from './AssistantApp.vue';
import type { AssistantStore } from './assistant-store.ts';
import {invoke} from '../bridge.ts';
import type { AsrStore } from '../ui/asr/store.ts';

vi.mock('../bridge.ts', () => ({ invoke: vi.fn(async () => {}) }));
vi.mock('./Orb.vue', () => ({ default: { template: '<span />' } }));
vi.stubGlobal('ResizeObserver', class { observe() {} disconnect() {} });

const wrappers: ReturnType<typeof mount>[] = [];
beforeEach(()=>vi.clearAllMocks());
afterEach(() => { wrappers.splice(0).forEach(w => w.unmount());document.body.replaceChildren(); });
function setup() {
  const store = reactive({ open: true, ball: false, phase: 'answering', hint: '', hintTab: '', question: '今天如何放松', answer: '可以先休息片刻。', working: '', failure: '', approval: null as null | { id: string; arguments: {}; tool: string }, ask: null, input: '草稿', status: '正在回答…', hide: vi.fn(), submit: vi.fn(), interrupt: vi.fn(), openChat: vi.fn(),toggleListening:vi.fn(),unfold:vi.fn() });
  const wrapper=mount(AssistantApp,{attachTo:document.body,props:{store:store as unknown as AssistantStore,asr:{level:0} as AsrStore},global:{stubs:{MarkdownView:true}}});
  wrappers.push(wrapper);
  return {store,wrapper};
}

describe('assistant response card', () => {
  it('closes only the upper card while leaving the capsule, draft and running reply untouched', async () => {
    const {store,wrapper}=setup();
    await wrapper.get('[data-testid="assistant-card-close"]').trigger('click');
    expect(wrapper.find('[data-testid="assistant-card"]').exists()).toBe(false);
    expect(wrapper.find('[data-testid="assistant"]').exists()).toBe(true);
    expect(wrapper.get<HTMLInputElement>('[data-testid="assistant-input"]').element.value).toBe('草稿');
    expect(store.hide).not.toHaveBeenCalled();
    expect(store.interrupt).not.toHaveBeenCalled();
    store.answer += '正在继续回答';await nextTick();
    expect(wrapper.find('[data-testid="assistant-card"]').exists()).toBe(false);
    store.question='新的问题';await nextTick();
    expect(wrapper.find('[data-testid="assistant-card"]').exists()).toBe(true);
  });

  it('reveals a fresh approval even after the card was dismissed', async () => {
    const {store,wrapper}=setup();
    await wrapper.get('[data-testid="assistant-card-close"]').trigger('click');
    store.approval={id:'approval1',arguments:{},tool:'shell'};await nextTick();
    expect(wrapper.find('[data-testid="assistant-approval"]').exists()).toBe(true);
  });
});


function move(x:number,y:number,buttons=1){window.dispatchEvent(new PointerEvent('pointermove',{pointerId:1,buttons,clientX:x,clientY:y}));}
function finish(){window.dispatchEvent(new Event('assistant-drag-end'));}
describe('assistant drag gestures',()=>{
 it('keeps a small orb movement as a click',async()=>{const {store,wrapper}=setup();const orb=wrapper.get('[data-testid="assistant-orb"]');await orb.trigger('pointerdown',{button:0,pointerId:1,clientX:10,clientY:10});move(13,12);window.dispatchEvent(new PointerEvent('pointerup',{pointerId:1}));await orb.trigger('click',{detail:1});expect(invoke).not.toHaveBeenCalledWith('assistant_drag');expect(store.toggleListening).toHaveBeenCalledOnce();});
 it('drags the orb after the threshold without toggling voice or losing the draft',async()=>{const {store,wrapper}=setup();const orb=wrapper.get('[data-testid="assistant-orb"]');await orb.trigger('pointerdown',{button:0,pointerId:1,clientX:10,clientY:10});move(18,10);await nextTick();expect(invoke).toHaveBeenCalledWith('assistant_drag');expect(wrapper.get('.stage').classes()).toContain('dragging');await orb.trigger('click',{detail:1});expect(store.toggleListening).not.toHaveBeenCalled();expect(store.interrupt).not.toHaveBeenCalled();expect(store.input).toBe('草稿');finish();await nextTick();expect(wrapper.get('.stage').classes()).not.toContain('dragging');});
 it('keeps text selection and control presses out of window dragging',async()=>{const {wrapper}=setup();for(const selector of ['[data-testid="assistant-input"]','.card-body','[data-testid="assistant-expand"]','[data-testid="assistant-copy"]']){await wrapper.get(selector).trigger('pointerdown',{button:0,pointerId:1,clientX:10,clientY:10});move(100,100);window.dispatchEvent(new PointerEvent('pointerup',{pointerId:1}));}expect(invoke).not.toHaveBeenCalledWith('assistant_drag');});
 it('uses the same threshold on the capsule edge and the folded ball',async()=>{const {store,wrapper}=setup();await wrapper.get('.capsule').trigger('pointerdown',{button:0,pointerId:1,clientX:10,clientY:10});move(17,10);expect(invoke).toHaveBeenCalledWith('assistant_drag');finish();store.open=false;store.ball=true;await nextTick();vi.mocked(invoke).mockClear();const ball=wrapper.get('[data-testid="assistant-ball"]');await ball.trigger('pointerdown',{button:0,pointerId:1,clientX:10,clientY:10});move(18,10);await ball.trigger('click',{detail:1});expect(invoke).toHaveBeenCalledWith('assistant_drag');expect(store.unfold).not.toHaveBeenCalled();finish();});
 it('does not treat a new input click after dragging as part of the drag',async()=>{const {wrapper}=setup();await wrapper.get('[data-testid="assistant-orb"]').trigger('pointerdown',{button:0,pointerId:1,clientX:10,clientY:10});move(20,10);finish();const field=wrapper.get('[data-testid="assistant-input"]');await field.trigger('pointerdown',{button:0,pointerId:2});const event=new MouseEvent('click',{bubbles:true,cancelable:true,detail:1});field.element.dispatchEvent(event);expect(event.defaultPrevented).toBe(false);});
});
describe('copying assistant replies',()=>{
 it('copies only the current AI response including markdown and confirms success',async()=>{const writeText=vi.fn().mockResolvedValue(undefined);Object.defineProperty(navigator,'clipboard',{value:{writeText},configurable:true});const {store,wrapper}=setup();store.answer='**建议**\n\n先休息片刻。';await nextTick();await wrapper.get('[data-testid="assistant-copy"]').trigger('click');await flushPromises();expect(writeText).toHaveBeenCalledWith('**建议**\n\n先休息片刻。');expect(wrapper.get('[data-testid="assistant-copy"]').text()).toContain('已复制');expect(store.input).toBe('草稿');expect(store.interrupt).not.toHaveBeenCalled();});
 it('shows failure and allows retry without changing the reply',async()=>{const writeText=vi.fn().mockRejectedValueOnce(Error('clipboard denied')).mockResolvedValue(undefined);Object.defineProperty(navigator,'clipboard',{value:{writeText},configurable:true});const {store,wrapper}=setup();const answer=store.answer;await wrapper.get('[data-testid="assistant-copy"]').trigger('click');await flushPromises();expect(wrapper.get('[data-testid="assistant-copy"]').text()).toContain('复制失败');expect(store.answer).toBe(answer);await wrapper.get('[data-testid="assistant-copy"]').trigger('click');await flushPromises();expect(wrapper.get('[data-testid="assistant-copy"]').text()).toContain('已复制');});
 it('hides the copy action when there is no AI response',async()=>{const {store,wrapper}=setup();store.answer='';await nextTick();expect(wrapper.find('[data-testid="assistant-copy"]').exists()).toBe(false);});
});
