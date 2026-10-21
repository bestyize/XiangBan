// @vitest-environment happy-dom
import { mount,flushPromises } from '@vue/test-utils';
import { reactive, nextTick } from 'vue';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import AssistantApp from './AssistantApp.vue';
import type { AssistantStore } from './assistant-store.ts';
import {invoke,listen} from '../bridge.ts';
import type { AsrStore } from '../ui/asr/store.ts';

vi.mock('../bridge.ts', () => ({ invoke: vi.fn(async () => {}), listen: vi.fn(async () => () => {}) }));
vi.mock('./Orb.vue', () => ({ default: { template: '<span />' } }));
vi.stubGlobal('ResizeObserver', class { observe() {} disconnect() {} });

const wrappers: ReturnType<typeof mount>[] = [];
beforeEach(()=>{vi.clearAllMocks();vi.mocked(invoke).mockReset().mockResolvedValue(undefined);});
afterEach(() => { wrappers.splice(0).forEach(w => w.unmount());document.body.replaceChildren();vi.useRealTimers(); });
function setup() {
  const store = reactive({ open: true, ball: false, phase: 'answering', hint: '', hintTab: '', question: '今天如何放松', answer: '可以先休息片刻。', working: '', failure: '', approval: null as null | { id: string; arguments: {}; tool: string }, ask: null, attachments: [] as string[], questionImages: [] as string[], prepareImages: vi.fn(), inputStarted: vi.fn(), note: vi.fn(), input: '草稿', status: '正在回答…', hide: vi.fn(), submit: vi.fn(), interrupt: vi.fn(), openChat: vi.fn(),newConversation:vi.fn(),openSettings:vi.fn(),toggleListening:vi.fn(),unfold:vi.fn() });
  const wrapper=mount(AssistantApp,{attachTo:document.body,props:{store:store as unknown as AssistantStore,asr:{level:0} as AsrStore},global:{stubs:{MarkdownView:true}}});
  wrappers.push(wrapper);
  return {store,wrapper};
}

async function openToolbar(wrapper: ReturnType<typeof setup>['wrapper']) {
  await wrapper.get('[data-testid="assistant-primary"]').trigger('keydown', { key: 'ArrowUp' });
  await nextTick();
}

describe('assistant response card', () => {
  it('treats edits and IME composition as input, while focus alone keeps reading', async () => {
    const {store,wrapper}=setup(); store.phase='speaking';
    const field=wrapper.get('[data-testid="assistant-input"]');
    await field.trigger('focus'); expect(store.inputStarted).not.toHaveBeenCalled();
    await field.trigger('beforeinput'); expect(store.inputStarted).toHaveBeenCalledOnce();
    await field.trigger('compositionstart'); expect(store.inputStarted).toHaveBeenCalledTimes(2);
    await field.trigger('compositionend'); store.inputStarted.mockClear(); await field.setValue('新的问题');
    expect(store.input).toBe('新的问题'); expect(store.inputStarted).toHaveBeenCalledOnce();
  });
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
 it('keeps text selection and control presses out of window dragging',async()=>{const {wrapper}=setup();await openToolbar(wrapper);for(const selector of ['[data-testid="assistant-input"]','.card-body','[data-testid="assistant-expand"]','[data-testid="assistant-copy"]']){await openToolbar(wrapper);await wrapper.get(selector).trigger('pointerdown',{button:0,pointerId:1,clientX:10,clientY:10});move(100,100);window.dispatchEvent(new PointerEvent('pointerup',{pointerId:1}));}expect(invoke).not.toHaveBeenCalledWith('assistant_drag');});
 it('uses the same threshold on the capsule edge and the folded ball',async()=>{const {store,wrapper}=setup();await wrapper.get('.capsule').trigger('pointerdown',{button:0,pointerId:1,clientX:10,clientY:10});move(17,10);expect(invoke).toHaveBeenCalledWith('assistant_drag');finish();store.open=false;store.ball=true;await nextTick();vi.mocked(invoke).mockClear();const ball=wrapper.get('[data-testid="assistant-ball"]');await ball.trigger('pointerdown',{button:0,pointerId:1,clientX:10,clientY:10});move(18,10);await ball.trigger('click',{detail:1});expect(invoke).toHaveBeenCalledWith('assistant_drag');expect(store.unfold).not.toHaveBeenCalled();finish();});
 it('never starts dragging from transparent headroom or the gap around the surfaces',async()=>{const {wrapper}=setup();for(const selector of ['.stage','.inner']){await wrapper.get(selector).trigger('pointerdown',{button:0,pointerId:1,clientX:30,clientY:20});move(80,80);window.dispatchEvent(new PointerEvent('pointerup',{pointerId:1}));}expect(invoke).not.toHaveBeenCalledWith('assistant_drag');});
 it('does not treat a new input click after dragging as part of the drag',async()=>{const {wrapper}=setup();await wrapper.get('[data-testid="assistant-orb"]').trigger('pointerdown',{button:0,pointerId:1,clientX:10,clientY:10});move(20,10);finish();const field=wrapper.get('[data-testid="assistant-input"]');await field.trigger('pointerdown',{button:0,pointerId:2});const event=new MouseEvent('click',{bubbles:true,cancelable:true,detail:1});field.element.dispatchEvent(event);expect(event.defaultPrevented).toBe(false);});
});
describe('copying assistant replies',()=>{
 it('copies only the current AI response including markdown and confirms success',async()=>{const writeText=vi.fn().mockResolvedValue(undefined);Object.defineProperty(navigator,'clipboard',{value:{writeText},configurable:true});const {store,wrapper}=setup();store.answer='**建议**\n\n先休息片刻。';await nextTick();await wrapper.get('[data-testid="assistant-copy"]').trigger('click');await flushPromises();expect(writeText).toHaveBeenCalledWith('**建议**\n\n先休息片刻。');expect(wrapper.get('[data-testid="assistant-copy"]').text()).toContain('已复制');expect(store.input).toBe('草稿');expect(store.interrupt).not.toHaveBeenCalled();});
 it('shows failure and allows retry without changing the reply',async()=>{const writeText=vi.fn().mockRejectedValueOnce(Error('clipboard denied')).mockResolvedValue(undefined);Object.defineProperty(navigator,'clipboard',{value:{writeText},configurable:true});const {store,wrapper}=setup();const answer=store.answer;await wrapper.get('[data-testid="assistant-copy"]').trigger('click');await flushPromises();expect(wrapper.get('[data-testid="assistant-copy"]').text()).toContain('复制失败');expect(store.answer).toBe(answer);await wrapper.get('[data-testid="assistant-copy"]').trigger('click');await flushPromises();expect(wrapper.get('[data-testid="assistant-copy"]').text()).toContain('已复制');});
 it('hides the copy action when there is no AI response',async()=>{const {store,wrapper}=setup();store.answer='';await nextTick();expect(wrapper.find('[data-testid="assistant-copy"]').exists()).toBe(false);});
});

const imageFile = () => new File(['test image'], 'photo.png', { type: 'image/png' });
describe('assistant image input', () => {
  it('opens the image picker from the compact toolbar without dragging', async () => {
    const { store, wrapper } = setup(); store.phase = 'idle'; await nextTick();
    const picker = wrapper.get<HTMLInputElement>('[data-testid="assistant-image-input"]');
    const click = vi.spyOn(picker.element, 'click').mockImplementation(() => {});
    await openToolbar(wrapper);
    const button = wrapper.get('[data-testid="assistant-add-image"]');
    expect(button.element.parentElement?.getAttribute('role')).toBe('toolbar');
    await button.trigger('pointerdown', { button: 0, pointerId: 1 }); move(80, 80);
    await button.trigger('click');
    expect(click).toHaveBeenCalledOnce(); expect(store.prepareImages).toHaveBeenCalledOnce();
    expect(invoke).not.toHaveBeenCalledWith('assistant_drag');
  });
  it.each(['drop', 'paste', 'pick'])('stages a %s image and lets the user remove it', async method => {
    const { store, wrapper } = setup(); store.phase = 'idle'; await nextTick();
    const file = imageFile();
    if (method === 'drop') await wrapper.get('.composer').trigger('drop', { dataTransfer: { files: [file], types: ['Files'] } });
    if (method === 'paste') await wrapper.get('.composer').trigger('paste', { clipboardData: { files: [file] } });
    if (method === 'pick') {
      const picker = wrapper.get<HTMLInputElement>('[data-testid="assistant-image-input"]');
      Object.defineProperty(picker.element, 'files', { value: [file], configurable: true });
      await picker.trigger('change');
    }
    await vi.waitFor(() => expect(store.attachments).toHaveLength(1)); await nextTick();
    expect(store.attachments[0]).toMatch(/^data:image\/png;base64,/);
    expect(wrapper.get('[data-testid="assistant-attachments"] img').attributes('src')).toBe(store.attachments[0]);
    await wrapper.get('[aria-label="移除附件 1"]').trigger('click'); expect(store.attachments).toEqual([]);
  });
  it('allows image-only questions and rejects unsupported and oversized files', async () => {
    const { store, wrapper } = setup(); store.phase = 'idle'; store.input = ''; store.attachments = ['data:image/png;base64,AA==']; await nextTick();
    await wrapper.get('[data-testid="assistant-primary"]').trigger('click'); expect(store.submit).toHaveBeenCalledOnce();
    await wrapper.get('.composer').trigger('drop', { dataTransfer: { files: [new File(['x'], 'x.svg', { type: 'image/svg+xml' })], types: ['Files'] } });
    await flushPromises(); expect(store.attachments).toHaveLength(1); expect(store.note).toHaveBeenCalledWith(expect.stringContaining('暂不支持这种文件'));
    const large = imageFile(); Object.defineProperty(large, 'size', { value: 3 * 1024 * 1024 + 1 });
    await wrapper.get('.composer').trigger('drop', { dataTransfer: { files: [large], types: ['Files'] } }); await flushPromises();
    expect(store.attachments).toHaveLength(1); expect(store.note).toHaveBeenCalledWith(expect.stringContaining('3MB'));
  });
  it('does not attach images while answering or after the capsule closes during reading', async () => {
    const { store, wrapper } = setup();
    await openToolbar(wrapper);
    expect(wrapper.get<HTMLButtonElement>('[data-testid="assistant-add-image"]').element.disabled).toBe(true);
    await wrapper.get('.composer').trigger('drop', { dataTransfer: { files: [imageFile()], types: ['Files'] } });
    expect(store.attachments).toHaveLength(0);
    store.phase = 'idle'; await nextTick();
    const read = vi.spyOn(FileReader.prototype, 'readAsDataURL').mockImplementation(function (this: FileReader) { setTimeout(() => this.onload?.(new ProgressEvent('load') as ProgressEvent<FileReader>), 10); });
    await wrapper.get('.composer').trigger('drop', { dataTransfer: { files: [imageFile()], types: ['Files'] } });
    store.open = false; await nextTick(); await new Promise(resolve => setTimeout(resolve, 20));
    expect(store.attachments).toEqual([]); read.mockRestore();
  });
});

describe('compact horizontal assistant tools', () => {
  it('keeps one right-hand button and shows the exact six actions in order', async () => {
    const { store, wrapper } = setup(); store.phase='idle'; store.input=''; await nextTick();
    expect(wrapper.find('[data-testid="assistant-toolbar"]').exists()).toBe(false);
    expect(wrapper.get('.capsule').findAll('button')).toHaveLength(2); // orb and one primary button
    await wrapper.get('[data-testid="assistant-primary"]').trigger('click');
    expect(wrapper.get('[role="toolbar"]').findAll('button').map(b=>b.attributes('aria-label'))).toEqual(['去超级小伴设置页','收起小伴','新会话','去聊天详情页','选择文件','截图']);
    await wrapper.get('[data-testid="assistant-settings"]').trigger('click'); await flushPromises();
    expect(store.openSettings).toHaveBeenCalledWith('assistant');
    await openToolbar(wrapper); await wrapper.get('[data-testid="assistant-expand"]').trigger('click');
    expect(store.openChat).toHaveBeenCalledOnce();
    await openToolbar(wrapper);await wrapper.get('[data-testid="assistant-new-chat"]').trigger('click');await flushPromises();expect(store.newConversation).toHaveBeenCalledOnce();
    await openToolbar(wrapper); await wrapper.get('[data-testid="assistant-close"]').trigger('click');
    expect(store.hide).toHaveBeenCalledOnce();
  });
  it('opens after a short hover and allows crossing into the toolbar before closing', async () => {
    vi.useFakeTimers(); const { wrapper }=setup();
    await wrapper.get('.tools-host').trigger('pointerenter'); await vi.advanceTimersByTimeAsync(219);
    expect(wrapper.find('[role="toolbar"]').exists()).toBe(false);
    await vi.advanceTimersByTimeAsync(1); expect(wrapper.find('[role="toolbar"]').exists()).toBe(true);
    await wrapper.get('.tools-host').trigger('pointerleave'); await vi.advanceTimersByTimeAsync(150);
    await wrapper.get('[role="toolbar"]').trigger('pointerenter'); await vi.advanceTimersByTimeAsync(500);
    expect(wrapper.find('[role="toolbar"]').exists()).toBe(true);
    await wrapper.get('.tools-host').trigger('pointerleave'); await vi.advanceTimersByTimeAsync(400);
    expect(wrapper.find('[role="toolbar"]').exists()).toBe(false);
  });
  it.each(['thinking','answering','speaking'])('keeps stop enabled during %s even before submit has returned',async phase=>{
    const {store,wrapper}=setup();store.phase='idle';let finish!:()=>void;
    store.submit.mockImplementationOnce(()=>{store.phase=phase;return new Promise<void>(resolve=>{finish=resolve;});});
    await nextTick();await wrapper.get('[data-testid="assistant-primary"]').trigger('click');await nextTick();
    const button=wrapper.get<HTMLButtonElement>('[data-testid="assistant-primary"]');
    expect(button.attributes('aria-label')).toBe('停止');expect(button.element.disabled).toBe(false);
    await button.trigger('click');expect(store.interrupt).toHaveBeenCalledOnce();expect(store.submit).toHaveBeenCalledOnce();
    finish();await flushPromises();
  });
  it('keeps send and stop as primary clicks while hovered tools are available', async () => {
    const {store,wrapper}=setup(); store.phase='idle'; await nextTick(); await openToolbar(wrapper);
    await wrapper.get('[data-testid="assistant-primary"]').trigger('click'); await flushPromises();
    expect(store.submit).toHaveBeenCalledOnce(); expect(wrapper.find('[role="toolbar"]').exists()).toBe(false);
    store.phase='answering'; await nextTick(); await wrapper.get('[data-testid="assistant-primary"]').trigger('click');
    expect(store.interrupt).toHaveBeenCalledOnce();
  });
  it('supports keyboard navigation and closes tools on Esc without hiding the assistant', async () => {
    const {store,wrapper}=setup(); await openToolbar(wrapper);
    expect(document.activeElement?.getAttribute('data-testid')).toBe('assistant-settings');
    await wrapper.get('[role="toolbar"]').trigger('keydown',{key:'ArrowRight'});
    expect(document.activeElement?.getAttribute('data-testid')).toBe('assistant-close');
    await wrapper.get('[role="toolbar"]').trigger('keydown',{key:'End'});
    expect(document.activeElement?.getAttribute('data-testid')).toBe('assistant-expand'); // uploads disabled while answering
    await wrapper.get('.stage').trigger('keydown',{key:'Escape'});
    expect(wrapper.find('[role="toolbar"]').exists()).toBe(false); expect(store.hide).not.toHaveBeenCalled();
    await wrapper.get('.stage').trigger('keydown',{key:'Escape'}); expect(store.hide).toHaveBeenCalledOnce();
  });
  it('gives a streamed answer a fixed body height so growth scrolls instead of resizing the window', async () => {
    const {store,wrapper}=setup();await nextTick();
    expect(wrapper.get('.card-body').classes()).toContain('answer-body');
    store.answer='';await nextTick();
    expect(wrapper.get('.card-body').classes()).not.toContain('answer-body');
  });
  it('reserves toolbar space once and never resizes while the toolbar animates or closes', async () => {
    vi.useFakeTimers();
    const bounds=vi.spyOn(HTMLElement.prototype,'getBoundingClientRect').mockImplementation(function(this:HTMLElement){
      return {top:this.classList.contains('tools-toolbar')?-50:0,height:this.classList.contains('inner')?92:0,bottom:92,left:0,right:480,width:480,x:0,y:0,toJSON(){}} as DOMRect;
    });
    try {
      const {store,wrapper}=setup();store.answer='';store.question='';store.phase='idle';
      await nextTick();await vi.advanceTimersByTimeAsync(30);
      expect(invoke).toHaveBeenCalledWith('assistant_fit',{height:156});
      vi.mocked(invoke).mockClear();
      await wrapper.get('.tools-host').trigger('pointerenter');await vi.advanceTimersByTimeAsync(240);
      expect(wrapper.find('[role="toolbar"]').exists()).toBe(true);
      expect(vi.mocked(invoke).mock.calls.filter(([name])=>name==='assistant_fit')).toHaveLength(0);
      await wrapper.get('.tools-host').trigger('pointerleave');await vi.advanceTimersByTimeAsync(450);
      expect(wrapper.find('[role="toolbar"]').exists()).toBe(false);
      expect(vi.mocked(invoke).mock.calls.filter(([name])=>name==='assistant_fit')).toHaveLength(0);
    } finally {bounds.mockRestore();}
  });
});
describe('assistant region screenshots',()=>{
  it('pauses voice and attaches a screenshot without sending or dropping the typed draft',async()=>{
    const {store,wrapper}=setup();store.phase='idle';store.attachments=['existing'];await nextTick();
    vi.mocked(invoke).mockImplementation(async command=>command==='assistant_screenshot'?'data:image/jpeg;base64,/9j/':undefined);
    await openToolbar(wrapper);await wrapper.get('[data-testid="assistant-screenshot"]').trigger('click');await flushPromises();
    expect(store.prepareImages).toHaveBeenCalledOnce();expect(store.attachments).toEqual(['existing','data:image/jpeg;base64,/9j/']);expect(store.input).toBe('草稿');expect(store.submit).not.toHaveBeenCalled();
  });
  it.each([null,Error('请允许屏幕录制')])('preserves the whole draft on cancellation or permission failure: %s',async result=>{
    const {store,wrapper}=setup();store.phase='idle';store.attachments=['existing'];await nextTick();
    vi.mocked(invoke).mockImplementation(async command=>{if(command==='assistant_screenshot'){if(result instanceof Error)throw result;return result;}});
    await openToolbar(wrapper);await wrapper.get('[data-testid="assistant-screenshot"]').trigger('click');await flushPromises();
    expect(store.attachments).toEqual(['existing']);expect(store.input).toBe('草稿');expect(store.submit).not.toHaveBeenCalled();
    if(result instanceof Error)expect(store.note).toHaveBeenCalledWith('请允许屏幕录制', '');
    expect(wrapper.get<HTMLButtonElement>('[data-testid="assistant-primary"]').element.disabled).toBe(false);
  });
  it('discards a late screenshot after closing and prevents duplicate capture requests',async()=>{
    const {store,wrapper}=setup();store.phase='idle';await nextTick();let finish!:(value:string)=>void;
    vi.mocked(invoke).mockImplementation(async command=>{if(command==='assistant_screenshot')return await new Promise<string>(resolve=>{finish=resolve;});});
    await openToolbar(wrapper);await wrapper.get('[data-testid="assistant-screenshot"]').trigger('click');await flushPromises();
    expect(wrapper.get<HTMLButtonElement>('[data-testid="assistant-primary"]').element.disabled).toBe(true);
    expect(vi.mocked(invoke).mock.calls.filter(([command])=>command==='assistant_screenshot')).toHaveLength(1);
    store.open=false;await nextTick();finish('data:image/jpeg;base64,/9j/');await flushPromises();expect(store.attachments).toEqual([]);
  });
  it('disables screenshot at the eight-image limit',async()=>{
    const {store,wrapper}=setup();store.phase='idle';store.attachments=Array(8).fill('image');await nextTick();await openToolbar(wrapper);
    expect(wrapper.get<HTMLButtonElement>('[data-testid="assistant-screenshot"]').element.disabled).toBe(true);
    expect(wrapper.get<HTMLButtonElement>('[data-testid="assistant-add-image"]').element.disabled).toBe(true);
  });
});

describe('assistant rainbow presentation',()=>{
  it('keeps the collapsed orb idle while a background answer is active',async()=>{
    const {store,wrapper}=setup();store.open=false;store.ball=true;store.phase='speaking';await nextTick();
    expect(wrapper.get('[data-testid="assistant-ball"]').attributes('data-phase')).toBe('idle');
    expect(wrapper.find('[data-testid="assistant-primary"]').exists()).toBe(false);
  });
  it.each(['thinking','answering','speaking'])('uses a dedicated stop style during %s and restores send or plus afterwards',async phase=>{
    const {store,wrapper}=setup();store.phase=phase;await nextTick();
    const button=wrapper.get('[data-testid="assistant-primary"]');expect(button.classes()).toContain('is-stop');
    await button.trigger('click');expect(store.interrupt).toHaveBeenCalledOnce();
    store.phase='idle';await nextTick();expect(button.classes()).not.toContain('is-stop');expect(button.attributes('aria-label')).toBe('发送');
    store.input='';await nextTick();expect(button.attributes('aria-label')).toBe('展开工具');
  });
});

describe('assistant native morph coordination',()=>{
  it('acknowledges the outgoing shape before preparing and revealing the incoming shape',async()=>{
    const animation=vi.spyOn(HTMLElement.prototype,'animate').mockReturnValue({finished:Promise.resolve(),cancel:vi.fn()} as unknown as Animation);
    const {store,wrapper}=setup();await flushPromises();
    const callback=vi.mocked(listen).mock.calls.find(([event])=>event==='assistant-transition')![1] as (value:any)=>void;
    callback({id:1,target:'ball',step:'leave',right:true});await flushPromises();
    expect(wrapper.get('.stage').classes()).toContain('morphing');
    expect(invoke).toHaveBeenCalledWith('assistant_transition_ready',{id:1,step:'leave'});
    store.open=false;store.ball=true;await nextTick();callback({id:1,target:'ball',step:'enter',right:true});await flushPromises();
    expect(invoke).toHaveBeenCalledWith('assistant_transition_ready',{id:1,step:'enter'});
    callback({id:1,target:'ball',step:'show',right:true});await flushPromises();expect(wrapper.get('.stage').classes()).not.toContain('morphing');animation.mockRestore();
  });
});


it('accepts a document drop in the capsule and renders a removable named card', async () => {
  const {store,wrapper}=setup();store.phase='idle';await nextTick();
  await wrapper.get('.composer').trigger('drop',{dataTransfer:{files:[new File(['amount\n12'],'预算.csv',{type:'text/csv'})],types:['Files']}});
  await vi.waitFor(()=>expect(store.attachments).toHaveLength(1));
  expect(store.attachments[0]).toMatchObject({name:'预算.csv',dataUrl:expect.stringMatching(/^data:text\/csv;base64,/)});
  expect(wrapper.get('[data-testid="assistant-attachments"]').text()).toContain('预算.csv');
  await wrapper.get('[aria-label="移除附件 1"]').trigger('click');expect(store.attachments).toHaveLength(0);
});
