// @vitest-environment happy-dom
import { mount,flushPromises } from '@vue/test-utils';
import { reactive, nextTick } from 'vue';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import AssistantApp from './AssistantApp.vue';
import type { AssistantStore } from './assistant-store.ts';
import {invoke} from '../bridge.ts';
import type { AsrStore } from '../ui/asr/store.ts';

vi.mock('../bridge.ts', () => ({ invoke: vi.fn(async () => {}), listen: vi.fn(async () => () => {}) }));
vi.mock('./Orb.vue', () => ({ default: { template: '<span />' } }));
vi.stubGlobal('ResizeObserver', class { observe() {} disconnect() {} });

const wrappers: ReturnType<typeof mount>[] = [];
beforeEach(()=>{vi.clearAllMocks();vi.mocked(invoke).mockReset().mockResolvedValue(undefined);});
afterEach(() => { wrappers.splice(0).forEach(w => w.unmount());document.body.replaceChildren();vi.useRealTimers(); });
const conversation = () => [
  { kind: 'user', id: 'u1', text: '今天如何放松', images: [] as string[] },
  { kind: 'assistant', id: 'a1', text: '可以先休息片刻。', streaming: false },
];
function setup(overrides: Record<string, unknown> = {}, stubs: Record<string, unknown> = {}) {
  const store = reactive({ open: true, ball: false, phase: 'answering', hint: '', hintTab: '', question: '今天如何放松', answer: '可以先休息片刻。', working: '', failure: '', approval: null as null | { id: string; arguments: {}; tool: string }, ask: null as null | { id: string; question: string; options: string[] },
    items: conversation() as unknown[], title: '放松一下', currentId: 's1' as string | null, recent: [{ id: 's1', title: '放松一下', createdAt: 1, updatedAt: 2 }, { id: 's2', title: '周报整理', createdAt: 1, updatedAt: 1 }],
    unread: false, notice: null as null | { kind: string; text: string }, dismissNotice: vi.fn(), switchSession: vi.fn(async () => {}), retry: vi.fn(async () => {}), decide: vi.fn(), reply: vi.fn(),
    attachments: [] as unknown[], questionImages: [] as string[], prepareImages: vi.fn(), inputStarted: vi.fn(), note: vi.fn(), input: '草稿', status: '正在回答…', hide: vi.fn(), submit: vi.fn(), interrupt: vi.fn(), openChat: vi.fn(),newConversation:vi.fn(),openSettings:vi.fn(),toggleListening:vi.fn(),unfold:vi.fn(),
    layout: { ball: null as null | { x: number; y: number; size: number }, right: true }, prefs: { open: 'type', read: 'voice', hover: true }, peek: false, hovering: false, reanchoring: 0, reanchorReady: vi.fn(), engage: vi.fn(), saveAnswer: vi.fn(async () => true),
    voiceAvailable: false, voiceStatus: '', live: false, muted: false, liveSince: 0, liveStatus: '', startLive: vi.fn(async () => {}), endLive: vi.fn(), toggleMute: vi.fn(async () => {}), ...overrides });
  store.dismissNotice.mockImplementation(() => { store.notice = null; });
  const asr=reactive({level:0,enabled:true,microphoneSupported:true,listening:false,preparing:false,recordingRequested:false,microphoneError:''});
  const wrapper=mount(AssistantApp,{attachTo:document.body,props:{store:store as unknown as AssistantStore,asr:asr as AsrStore},global:{stubs:{MarkdownView:true,...stubs}}});
  wrappers.push(wrapper);
  return {store,asr,wrapper};
}

describe('assistant capsule', () => {
 it('uses actual capture state for the orb action and keeps typing available during preparation',async()=>{
   const {store,asr,wrapper}=setup();store.phase='idle';await nextTick();const orb=wrapper.get('[data-testid="assistant-orb"]');
   asr.enabled=false;await nextTick();expect(orb.attributes('aria-label')).toBe('开启语音输入');
   asr.enabled=true;asr.preparing=true;await nextTick();expect(orb.attributes('aria-label')).toBe('正在准备语音输入');expect(wrapper.get('[data-testid="assistant-input"]').attributes('disabled')).toBeUndefined();
   asr.preparing=false;asr.recordingRequested=true;await nextTick();expect(orb.attributes('aria-label')).toBe('取消语音启动');
   asr.listening=true;await nextTick();expect(orb.attributes('aria-label')).toBe('结束录音');
 });
  it.each(['enter','button'])('can send during readout using %s without stopping on edit',async method=>{
    const {store,wrapper}=setup();store.phase='speaking';store.input='';await nextTick();
    const field=wrapper.get('[data-testid="assistant-input"]');await field.setValue('新的问题');
    expect(store.interrupt).not.toHaveBeenCalled();expect(store.submit).not.toHaveBeenCalled();
    const button=wrapper.get('[data-testid="assistant-primary"]');expect(button.attributes('aria-label')).toBe('发送');expect(button.classes()).not.toContain('is-stop');
    if(method==='enter')await field.trigger('keydown',{key:'Enter'});else await button.trigger('click');
    expect(store.submit).toHaveBeenCalledOnce();expect(store.interrupt).not.toHaveBeenCalled();
  });
  it('keeps Shift+Enter as a new line instead of sending', async () => {
    const {store,wrapper}=setup();store.phase='idle';await nextTick();
    await wrapper.get('[data-testid="assistant-input"]').trigger('keydown',{key:'Enter',shiftKey:true});
    expect(store.submit).not.toHaveBeenCalled();
  });
  it('treats edits and IME composition as input, while focus alone keeps reading', async () => {
    const {store,wrapper}=setup(); store.phase='speaking';
    const field=wrapper.get('[data-testid="assistant-input"]');
    await field.trigger('focus'); expect(store.inputStarted).not.toHaveBeenCalled();
    await field.trigger('beforeinput'); expect(store.inputStarted).toHaveBeenCalledOnce();
    await field.trigger('compositionstart'); expect(store.inputStarted).toHaveBeenCalledTimes(2);
    await field.trigger('compositionend'); store.inputStarted.mockClear(); await field.setValue('新的问题');
    expect(store.input).toBe('新的问题'); expect(store.inputStarted).toHaveBeenCalledOnce();
  });
  it('keeps the orb, the text box, add, screenshot and send on the capsule', async () => {
    const {store,wrapper}=setup();store.phase='idle';await nextTick();
    expect(wrapper.get('.capsule').findAll('button').map(b=>b.attributes('data-testid'))).toEqual(['assistant-orb','assistant-add-image','assistant-screenshot','assistant-primary']);
    expect(wrapper.find('[role="toolbar"]').exists()).toBe(false);
  });
  it('offers 实时对话 beside send, and turns the capsule into a voice bar', async () => {
    const {store,wrapper}=setup({input:'',phase:'idle',voiceAvailable:true});await nextTick();
    expect(wrapper.get('[data-testid="assistant-primary"]').attributes('aria-label')).toBe('发送');
    await wrapper.get('[data-testid="assistant-live-start"]').trigger('click');await flushPromises();
    expect(store.startLive).toHaveBeenCalledOnce();
    store.input='打字';await nextTick();
    expect(wrapper.find('[data-testid="assistant-live-start"]').exists()).toBe(true);
    expect(wrapper.get('[data-testid="assistant-primary"]').attributes('aria-label')).toBe('发送');
    store.phase='answering';await nextTick();
    expect(wrapper.get<HTMLButtonElement>('[data-testid="assistant-live-start"]').element.disabled).toBe(true);
    store.phase='idle';
    store.input='';store.live=true;store.liveStatus='在听';store.liveSince=Date.now();await nextTick();
    expect(wrapper.find('[data-testid="assistant-input"]').exists()).toBe(false);
    expect(wrapper.get('[data-testid="assistant-live-status"]').text()).toContain('在听');
    await wrapper.get('[data-testid="assistant-live-mute"]').trigger('click');expect(store.toggleMute).toHaveBeenCalledOnce();
    await wrapper.get('[data-testid="assistant-live-end"]').trigger('click');expect(store.endLive).toHaveBeenCalledOnce();
  });
  it('keeps the empty text box one line high and refits a draft when the window widens', async () => {
    const {store,wrapper}=setup({input:''});await nextTick();
    const field=wrapper.get('[data-testid="assistant-input"]').element as HTMLTextAreaElement;
    // A narrow window wraps the placeholder: its height must not be taken for the text's.
    let scroll=80;Object.defineProperty(field,'scrollHeight',{configurable:true,get:()=>scroll});
    window.dispatchEvent(new Event('resize'));expect(field.style.height).toBe('');
    store.input='一段比较长的草稿';await nextTick();await nextTick();expect(field.style.height).toBe('80px');
    scroll=36;window.dispatchEvent(new Event('resize'));expect(field.style.height).toBe('36px');
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
  it.each(['thinking','answering','speaking'])('uses a dedicated stop style during %s and a disabled send afterwards',async phase=>{
    const {store,wrapper}=setup();store.phase=phase;store.input='';await nextTick();
    const button=wrapper.get<HTMLButtonElement>('[data-testid="assistant-primary"]');expect(button.classes()).toContain('is-stop');
    await button.trigger('click');expect(store.interrupt).toHaveBeenCalledOnce();
    store.phase='idle';store.input='草稿';await nextTick();expect(button.classes()).not.toContain('is-stop');expect(button.attributes('aria-label')).toBe('发送');expect(button.element.disabled).toBe(false);
    store.input='';await nextTick();expect(button.element.disabled).toBe(true);
  });
  it('clears the text on the first Esc and folds away on the second', async () => {
    const {store,wrapper}=setup();store.phase='idle';await nextTick();
    await wrapper.get('.stage').trigger('keydown',{key:'Escape'});expect(store.input).toBe('');expect(store.hide).not.toHaveBeenCalled();
    await wrapper.get('.stage').trigger('keydown',{key:'Escape'});expect(store.hide).toHaveBeenCalledOnce();
  });
});

describe('assistant conversation card', () => {
  it('shows every turn of the open conversation', async () => {
    const {store,wrapper}=setup();
    store.items=[...conversation(),{kind:'user',id:'u2',text:'再具体一点',images:[]},{kind:'assistant',id:'a2',text:'试试深呼吸。',streaming:true}];await nextTick();
    expect(wrapper.findAll('.you').map(n=>n.text())).toEqual(['今天如何放松','再具体一点']);
    expect(wrapper.findAll('.reply')).toHaveLength(2);
    expect(wrapper.get('[data-testid="assistant-sessions"]').text()).toContain('放松一下');
  });
  it('gives a streaming answer a fixed body height so growth scrolls instead of resizing the window', async () => {
    const {store,wrapper}=setup();await nextTick();
    expect(wrapper.get('.card-body').classes()).toContain('fixed');
    store.phase='idle';await nextTick();expect(wrapper.get('.card-body').classes()).not.toContain('fixed');
    store.items=[];store.hint='还没有配置模型';await nextTick();
    expect(wrapper.get('.card-body').classes()).not.toContain('fixed');
  });
  it('opens on the capsule alone with quick actions when nothing waits', async () => {
    const {wrapper}=setup({open:false,phase:'idle',input:''});const store=wrapper.props('store') as unknown as {open:boolean};
    store.open=true;await nextTick();
    expect(wrapper.find('[data-testid="assistant-card"]').exists()).toBe(false);
    expect(wrapper.get('[data-testid="assistant-chips"]').text()).toContain('继续「放松一下」');
    await wrapper.get('[data-testid="assistant-chip-continue"]').trigger('click');
    expect(wrapper.find('[data-testid="assistant-card"]').exists()).toBe(true);
  });
  it('opens straight onto the card when an answer arrived while folded', async () => {
    const {store,wrapper}=setup({open:false,phase:'idle',input:''});
    store.unread=true;await nextTick();store.unread=false;store.open=true;await nextTick();
    expect(wrapper.find('[data-testid="assistant-card"]').exists()).toBe(true);
  });
  it('offers another recent conversation when the open one is empty', async () => {
    const {store,wrapper}=setup({open:false,phase:'idle',input:'',items:[],title:'',currentId:null});
    store.open=true;await nextTick();
    await wrapper.get('[data-testid="assistant-chip-recent"]').trigger('click');await flushPromises();
    expect(store.switchSession).toHaveBeenCalledWith('s1');
  });
  it('reveals a fresh approval and offers all three answers', async () => {
    const {store,wrapper}=setup({open:false,phase:'idle',input:''});store.open=true;await nextTick();
    store.approval={id:'approval1',arguments:{},tool:'shell'};store.phase='attention';await nextTick();
    expect(wrapper.find('[data-testid="assistant-approval"]').exists()).toBe(true);
    const buttons=wrapper.get('[data-testid="assistant-approval"]').findAll('button');
    expect(buttons.map(b=>b.text())).toEqual(['拒绝','本次会话都允许','允许这一次']);
    await buttons[1].trigger('click');expect(store.decide).toHaveBeenCalledWith('always');
  });
  it('switches conversations, starts a new one, continues in the main window and folds from the header', async () => {
    const {store,wrapper}=setup();store.phase='idle';await nextTick();
    await wrapper.get('[data-testid="assistant-sessions"]').trigger('click');
    const items=wrapper.get('[data-testid="assistant-session-menu"]').findAll('[role="menuitemradio"]');
    expect(items.map(i=>i.attributes('aria-checked'))).toEqual(['true','false']);
    await items[1].trigger('click');await flushPromises();expect(store.switchSession).toHaveBeenCalledWith('s2');
    await wrapper.get('[data-testid="assistant-sessions"]').trigger('click');
    await wrapper.get('[data-testid="assistant-settings"]').trigger('click');await flushPromises();expect(store.openSettings).toHaveBeenCalledWith('assistant');
    await wrapper.get('[data-testid="assistant-new-chat"]').trigger('click');await flushPromises();expect(store.newConversation).toHaveBeenCalledOnce();
    store.items=conversation();await nextTick();wrapper.vm.$forceUpdate();
  });
  it('shows recent conversations inside the card in place of the conversation, and brings it back', async () => {
    const {store,wrapper}=setup();store.phase='idle';await nextTick();
    const body=wrapper.get('[data-testid="assistant-card-body"]').element as HTMLElement;
    const menu=()=>wrapper.find('[data-testid="assistant-session-menu"]');
    await wrapper.get('[data-testid="assistant-sessions"]').trigger('click');
    // In the card's flow, not floating over the capsule below it.
    expect(menu().element.parentElement).toBe(wrapper.get('[data-testid="assistant-card"]').element);
    expect(body.style.display).toBe('none');
    // Pressing inside the list keeps it open; pressing elsewhere closes it and the conversation comes back.
    menu().element.dispatchEvent(new PointerEvent('pointerdown',{bubbles:true}));await nextTick();
    expect(menu().exists()).toBe(true);
    document.body.dispatchEvent(new PointerEvent('pointerdown',{bubbles:true}));await nextTick();
    expect(menu().exists()).toBe(false);expect(body.style.display).toBe('');
  });
  it('saves an answer to 资料库 once and shows it is kept', async () => {
    const {store,wrapper}=setup();store.phase='idle';await nextTick();
    const save=wrapper.get('[data-testid="assistant-save"]');
    expect(save.text()).toBe('存到资料库');
    await save.trigger('click');await flushPromises();
    expect(store.saveAnswer).toHaveBeenCalledWith('可以先休息片刻。');
    expect(save.text()).toBe('已收藏');
    await save.trigger('click');await flushPromises();
    expect(store.saveAnswer).toHaveBeenCalledOnce();
  });
  it('continues in the main window and folds away from the header', async () => {
    const {store,wrapper}=setup();store.phase='idle';await nextTick();
    await wrapper.get('[data-testid="assistant-expand"]').trigger('click');await flushPromises();expect(store.openChat).toHaveBeenCalledOnce();
    await wrapper.get('[data-testid="assistant-close"]').trigger('click');await flushPromises();expect(store.hide).toHaveBeenCalledOnce();
  });
  it('asks the last question again from the last answer', async () => {
    const {store,wrapper}=setup();store.phase='idle';await nextTick();
    await wrapper.get('[data-testid="assistant-retry"]').trigger('click');await flushPromises();expect(store.retry).toHaveBeenCalledOnce();
  });
});

describe('folding away on a click elsewhere', () => {
  const blur = () => { vi.spyOn(document,'hasFocus').mockReturnValue(false); window.dispatchEvent(new Event('blur')); };
  it('folds an idle, empty capsule away', async () => {
    vi.useFakeTimers();const {store}=setup({phase:'idle',input:''});await nextTick();
    blur();await vi.advanceTimersByTimeAsync(200);expect(store.hide).toHaveBeenCalledOnce();
  });
  it.each([
    ['a draft', { phase:'idle', input:'草稿' }],
    ['an answer being written', { phase:'answering', input:'' }],
    ['a question for the user', { phase:'attention', input:'' }],
    ['attachments', { phase:'idle', input:'', attachments:['data:image/png;base64,AA=='] }],
  ])('stays while there is %s', async (_name, state) => {
    vi.useFakeTimers();const {store}=setup(state);await nextTick();
    blur();await vi.advanceTimersByTimeAsync(200);expect(store.hide).not.toHaveBeenCalled();
  });
  it('stays while pinned', async () => {
    vi.useFakeTimers();const {store,wrapper}=setup({phase:'idle',input:''});await nextTick();
    await wrapper.get('[data-testid="assistant-chip-continue"]').trigger('click');
    await wrapper.get('[data-testid="assistant-pin"]').trigger('click');expect(wrapper.get('[data-testid="assistant-pin"]').attributes('aria-pressed')).toBe('true');
    blur();await vi.advanceTimersByTimeAsync(200);expect(store.hide).not.toHaveBeenCalled();
  });
});

const BALL={x:408,y:520,size:72};
const folded=(overrides:Record<string,unknown>={})=>setup({open:false,ball:true,phase:'idle',layout:{ball:BALL,right:true},...overrides});
describe('the folded ball', () => {
  it('sits where the host placed it and shows a ring while answering and a badge while waiting', async () => {
    const {store,wrapper}=folded({phase:'answering'});await nextTick();
    expect(wrapper.get('[data-testid="assistant-ball-stage"]').attributes('style')).toContain('left: 408px; top: 520px; width: 72px; height: 72px');
    const ball=wrapper.get('[data-testid="assistant-ball"]');
    expect(ball.attributes('data-phase')).toBe('answering');expect(wrapper.find('[data-testid="assistant-ball-ring"]').exists()).toBe(true);
    store.phase='attention';await nextTick();
    expect(wrapper.get('[data-testid="assistant-ball-badge"]').classes()).toContain('attention');
    store.phase='idle';store.unread=true;await nextTick();
    expect(wrapper.get('[data-testid="assistant-ball-badge"]').classes()).toContain('unread');
    expect(wrapper.find('[data-testid="assistant-primary"]').exists()).toBe(false);
  });
  it('is not drawn until the host says where it is', async () => {
    const {wrapper}=folded({layout:{ball:null,right:true}});await nextTick();
    expect(wrapper.find('[data-testid="assistant-ball-stage"]').exists()).toBe(false);
  });
  it('says a finished answer beside itself, toward the middle of the screen, for three seconds', async () => {
    vi.useFakeTimers();
    const {store,wrapper}=folded();await nextTick();
    store.notice={kind:'done',text:'周报初稿整理好了'};await nextTick();
    expect(wrapper.get('[data-testid="assistant-bubble"]').text()).toContain('周报初稿整理好了');
    expect(wrapper.get('[data-testid="assistant-ball-stage"]').classes()).toContain('bubble-left');
    // Drawn inside the stage: the window is not widened for it.
    expect(vi.mocked(invoke).mock.calls.map(([command])=>command)).not.toContain('assistant_ball_bubble');
    await wrapper.get('[data-testid="assistant-bubble"]').trigger('click');expect(store.unfold).toHaveBeenCalledOnce();
    await vi.advanceTimersByTimeAsync(3000);
    expect(store.notice).toBeNull();
  });
  it('keeps a waiting notice until it is seen', async () => {
    vi.useFakeTimers();
    const {store,wrapper}=folded({phase:'attention',layout:{ball:BALL,right:false}});await nextTick();
    store.notice={kind:'attention',text:'shell 需要你许可'};await nextTick();
    await vi.advanceTimersByTimeAsync(10000);
    expect(store.dismissNotice).not.toHaveBeenCalled();expect(wrapper.get('[data-testid="assistant-bubble"]').text()).toContain('去处理');
    expect(wrapper.get('[data-testid="assistant-ball-stage"]').classes()).not.toContain('bubble-left');
  });
});

describe('resting the pointer on the ball', () => {
  it('opens the capsule for a glance once the pointer rests, not while it passes', async () => {
    vi.useFakeTimers();
    const {store,wrapper}=folded();await nextTick();
    const ball=wrapper.get('[data-testid="assistant-ball"]');
    await ball.trigger('pointerenter',{clientX:10,clientY:10});await vi.advanceTimersByTimeAsync(200);
    await ball.trigger('pointermove',{clientX:40,clientY:12});await vi.advanceTimersByTimeAsync(200);
    expect(store.unfold).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(80);expect(store.unfold).toHaveBeenCalledWith(true);
  });
  it('waits for the pointer to leave after the capsule closes over it, and stays off when switched off', async () => {
    vi.useFakeTimers();
    const {store,wrapper}=setup({open:true,phase:'idle',layout:{ball:BALL,right:true}});await nextTick();
    store.open=false;store.ball=true;await nextTick();await flushPromises();
    const ball=wrapper.get('[data-testid="assistant-ball"]');
    await ball.trigger('pointerenter',{clientX:10,clientY:10});await vi.advanceTimersByTimeAsync(1000);
    expect(store.unfold).not.toHaveBeenCalled();
    await ball.trigger('pointerleave');await ball.trigger('pointerenter',{clientX:10,clientY:10});await vi.advanceTimersByTimeAsync(300);
    expect(store.unfold).toHaveBeenCalledWith(true);
    vi.mocked(store.unfold).mockClear();store.prefs.hover=false;
    await ball.trigger('pointerleave');await ball.trigger('pointerenter',{clientX:10,clientY:10});await vi.advanceTimersByTimeAsync(1000);
    expect(store.unfold).not.toHaveBeenCalled();
  });
  it('folds a glance away soon after the pointer leaves, but not once it has been used', async () => {
    vi.useFakeTimers();
    const {store,wrapper}=setup({open:true,phase:'idle',peek:true,hovering:true,layout:{ball:BALL,right:true}});await nextTick();
    store.hovering=false;await nextTick();await vi.advanceTimersByTimeAsync(599);expect(store.hide).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(1);expect(store.hide).toHaveBeenCalledOnce();
    vi.mocked(store.hide).mockClear();store.hovering=true;await nextTick();
    vi.mocked(store.engage).mockImplementation(()=>{store.peek=false;});
    await wrapper.get('[data-testid="assistant-input"]').trigger('pointerdown',{button:0});expect(store.engage).toHaveBeenCalledOnce();
    store.hovering=false;await nextTick();await vi.advanceTimersByTimeAsync(2000);expect(store.hide).not.toHaveBeenCalled();
  });
});

function move(x:number,y:number,buttons=1){window.dispatchEvent(new PointerEvent('pointermove',{pointerId:1,buttons,clientX:x,clientY:y}));}
function finish(){window.dispatchEvent(new Event('assistant-drag-end'));}
describe('assistant drag gestures',()=>{
 it('keeps a small orb movement as a click',async()=>{const {store,wrapper}=setup();const orb=wrapper.get('[data-testid="assistant-orb"]');await orb.trigger('pointerdown',{button:0,pointerId:1,clientX:10,clientY:10});move(13,12);window.dispatchEvent(new PointerEvent('pointerup',{pointerId:1}));await orb.trigger('click',{detail:1});expect(invoke).not.toHaveBeenCalledWith('assistant_drag');expect(store.toggleListening).toHaveBeenCalledOnce();});
 it('drags the orb after the threshold without toggling voice or losing the draft',async()=>{const {store,wrapper}=setup();const orb=wrapper.get('[data-testid="assistant-orb"]');await orb.trigger('pointerdown',{button:0,pointerId:1,clientX:10,clientY:10});move(18,10);await nextTick();expect(invoke).toHaveBeenCalledWith('assistant_drag');expect(wrapper.get('.stage').classes()).toContain('dragging');await orb.trigger('click',{detail:1});expect(store.toggleListening).not.toHaveBeenCalled();expect(store.interrupt).not.toHaveBeenCalled();expect(store.input).toBe('草稿');finish();await nextTick();expect(wrapper.get('.stage').classes()).not.toContain('dragging');});
 it('keeps text selection and control presses out of window dragging',async()=>{const {wrapper}=setup();for(const selector of ['[data-testid="assistant-input"]','.card-body','[data-testid="assistant-expand"]','[data-testid="assistant-screenshot"]']){await wrapper.get(selector).trigger('pointerdown',{button:0,pointerId:1,clientX:10,clientY:10});move(100,100);window.dispatchEvent(new PointerEvent('pointerup',{pointerId:1}));}expect(invoke).not.toHaveBeenCalledWith('assistant_drag');});
 it('uses the same threshold on the capsule edge and the folded ball',async()=>{const {store,wrapper}=setup({layout:{ball:BALL,right:true}});await wrapper.get('.capsule').trigger('pointerdown',{button:0,pointerId:1,clientX:10,clientY:10});move(17,10);expect(invoke).toHaveBeenCalledWith('assistant_drag');finish();store.open=false;store.ball=true;await nextTick();await flushPromises();vi.mocked(invoke).mockClear();const ball=wrapper.get('[data-testid="assistant-ball"]');await ball.trigger('pointerdown',{button:0,pointerId:1,clientX:10,clientY:10});move(18,10);await ball.trigger('click',{detail:1});expect(invoke).toHaveBeenCalledWith('assistant_drag');expect(store.unfold).not.toHaveBeenCalled();finish();});
 it('never starts dragging from transparent headroom or the gap around the surfaces',async()=>{const {wrapper}=setup();for(const selector of ['.stage','.inner']){await wrapper.get(selector).trigger('pointerdown',{button:0,pointerId:1,clientX:30,clientY:20});move(80,80);window.dispatchEvent(new PointerEvent('pointerup',{pointerId:1}));}expect(invoke).not.toHaveBeenCalledWith('assistant_drag');});
 it('does not treat a new input click after dragging as part of the drag',async()=>{const {wrapper}=setup();await wrapper.get('[data-testid="assistant-orb"]').trigger('pointerdown',{button:0,pointerId:1,clientX:10,clientY:10});move(20,10);finish();const field=wrapper.get('[data-testid="assistant-input"]');await field.trigger('pointerdown',{button:0,pointerId:2});const event=new MouseEvent('click',{bubbles:true,cancelable:true,detail:1});field.element.dispatchEvent(event);expect(event.defaultPrevented).toBe(false);});
});
describe('copying assistant replies',()=>{
 it('copies the answer including markdown and confirms success',async()=>{const writeText=vi.fn().mockResolvedValue(undefined);Object.defineProperty(navigator,'clipboard',{value:{writeText},configurable:true});const {store,wrapper}=setup();store.items=[conversation()[0],{kind:'assistant',id:'a1',text:'**建议**\n\n先休息片刻。',streaming:false}];await nextTick();await wrapper.get('[data-testid="assistant-copy"]').trigger('click');await flushPromises();expect(writeText).toHaveBeenCalledWith('**建议**\n\n先休息片刻。');expect(wrapper.get('[data-testid="assistant-copy"]').text()).toContain('已复制');expect(store.input).toBe('草稿');expect(store.interrupt).not.toHaveBeenCalled();});
 it('shows failure and allows retry without changing the reply',async()=>{const writeText=vi.fn().mockRejectedValueOnce(Error('clipboard denied')).mockResolvedValue(undefined);Object.defineProperty(navigator,'clipboard',{value:{writeText},configurable:true});const {wrapper}=setup();await wrapper.get('[data-testid="assistant-copy"]').trigger('click');await flushPromises();expect(wrapper.get('[data-testid="assistant-copy"]').text()).toContain('复制失败');await wrapper.get('[data-testid="assistant-copy"]').trigger('click');await flushPromises();expect(wrapper.get('[data-testid="assistant-copy"]').text()).toContain('已复制');});
 it('hides the copy action while the answer is still being written',async()=>{const {store,wrapper}=setup();store.items=[conversation()[0],{kind:'assistant',id:'a1',text:'正在',streaming:true}];await nextTick();expect(wrapper.find('[data-testid="assistant-copy"]').exists()).toBe(false);});
});

const imageFile = () => new File(['test image'], 'photo.png', { type: 'image/png' });
describe('assistant image input', () => {
  it('opens the file picker from the add button without dragging', async () => {
    const { store, wrapper } = setup(); store.phase = 'idle'; await nextTick();
    const picker = wrapper.get<HTMLInputElement>('[data-testid="assistant-image-input"]');
    const click = vi.spyOn(picker.element, 'click').mockImplementation(() => {});
    const button = wrapper.get('[data-testid="assistant-add-image"]');
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

describe('assistant region screenshots',()=>{
  it('pauses voice and attaches a screenshot without sending or dropping the typed draft',async()=>{
    const {store,wrapper}=setup();store.phase='idle';store.attachments=['existing'];await nextTick();
    vi.mocked(invoke).mockImplementation(async command=>command==='assistant_screenshot'?'data:image/jpeg;base64,/9j/':undefined);
    await wrapper.get('[data-testid="assistant-screenshot"]').trigger('click');await flushPromises();
    expect(store.prepareImages).toHaveBeenCalledOnce();expect(store.attachments).toEqual(['existing','data:image/jpeg;base64,/9j/']);expect(store.input).toBe('草稿');expect(store.submit).not.toHaveBeenCalled();
  });
  it.each([null,Error('请允许屏幕录制')])('preserves the whole draft on cancellation or permission failure: %s',async result=>{
    const {store,wrapper}=setup();store.phase='idle';store.attachments=['existing'];await nextTick();
    vi.mocked(invoke).mockImplementation(async command=>{if(command==='assistant_screenshot'){if(result instanceof Error)throw result;return result;}});
    await wrapper.get('[data-testid="assistant-screenshot"]').trigger('click');await flushPromises();
    expect(store.attachments).toEqual(['existing']);expect(store.input).toBe('草稿');expect(store.submit).not.toHaveBeenCalled();
    if(result instanceof Error)expect(store.note).toHaveBeenCalledWith('请允许屏幕录制', '');
    expect(wrapper.get<HTMLButtonElement>('[data-testid="assistant-primary"]').element.disabled).toBe(false);
  });
  it('discards a late screenshot after closing and prevents duplicate capture requests',async()=>{
    const {store,wrapper}=setup();store.phase='idle';await nextTick();let finish!:(value:string)=>void;
    vi.mocked(invoke).mockImplementation(async command=>{if(command==='assistant_screenshot')return await new Promise<string>(resolve=>{finish=resolve;});});
    await wrapper.get('[data-testid="assistant-screenshot"]').trigger('click');await flushPromises();
    expect(wrapper.get<HTMLButtonElement>('[data-testid="assistant-primary"]').element.disabled).toBe(true);
    await wrapper.get('[data-testid="assistant-screenshot"]').trigger('click');
    expect(vi.mocked(invoke).mock.calls.filter(([command])=>command==='assistant_screenshot')).toHaveLength(1);
    store.open=false;await nextTick();finish('data:image/jpeg;base64,/9j/');await flushPromises();expect(store.attachments).toEqual([]);
  });
  it('disables screenshot at the eight-image limit',async()=>{
    const {store,wrapper}=setup();store.phase='idle';store.attachments=Array(8).fill('image');await nextTick();
    expect(wrapper.get<HTMLButtonElement>('[data-testid="assistant-screenshot"]').element.disabled).toBe(true);
    expect(wrapper.get<HTMLButtonElement>('[data-testid="assistant-add-image"]').element.disabled).toBe(true);
  });
});

describe('ball and capsule in one window',()=>{
  function animations(){
    const runs:{el:Element;frames:Keyframe[];finish:()=>void;reverse:ReturnType<typeof vi.fn>}[]=[];
    const spy=vi.spyOn(HTMLElement.prototype,'animate').mockImplementation(function(this:HTMLElement,frames){
      let finish!:()=>void;const finished=new Promise<void>(resolve=>{finish=resolve;});
      const reverse=vi.fn();runs.push({el:this,frames:frames as Keyframe[],finish,reverse});
      return {finished,cancel:vi.fn(),reverse} as unknown as Animation;
    });
    return {runs,spy,finishAll:()=>runs.splice(0).forEach(run=>run.finish())};
  }
  it('grows the capsule out of the ball and back without moving the window',async()=>{
    const {runs,spy,finishAll}=animations();
    const {store,wrapper}=folded();await nextTick();
    store.ball=false;store.open=true;await nextTick();await nextTick();await flushPromises();
    expect(wrapper.get('.stage').classes()).toContain('morph-expand');
    const glass=runs.find(run=>run.el.classList.contains('morph-glass'))!;
    // From the ball's own disc: 44px in the middle of its 72px square.
    expect(glass.frames[0]).toMatchObject({left:'422px',top:'534px',width:'44px',height:'44px',opacity:0});
    expect(glass.frames[1]).toMatchObject({borderRadius:'26px',opacity:1});
    expect(runs.find(run=>run.el.classList.contains('morph-orb'))!.frames[0]).toEqual({transform:'translate(422px, 534px) scale(1)'});
    finishAll();await flushPromises();await nextTick();
    expect(wrapper.get('.stage').classes()).not.toContain('morph-expand');
    store.open=false;store.ball=true;await nextTick();
    // The capsule stays until the glass has shrunk back into the ball, which is hidden till then.
    expect(wrapper.get('.stage').classes()).toContain('morph-collapse');
    expect(wrapper.find('[data-testid="assistant"]').exists()).toBe(true);
    await flushPromises();expect(runs.find(run=>run.el.classList.contains('morph-glass'))!.frames[1]).toMatchObject({left:'422px',width:'44px',opacity:0});
    finishAll();await flushPromises();await nextTick();
    expect(wrapper.find('[data-testid="assistant"]').exists()).toBe(false);
    expect(wrapper.find('[data-testid="assistant-ball"]').exists()).toBe(true);
    // The stand-ins are gone in the same frame, so the ball shows at once: no fade-in, no blink.
    expect(wrapper.get('.stage').classes()).not.toContain('ball-fade');
    for (const command of ['assistant_fit','assistant_transition_ready']) expect(vi.mocked(invoke).mock.calls.map(([name])=>name)).not.toContain(command);
    spy.mockRestore();
  });
  it('takes the capsule away in the same frame the ball comes back, never showing it alone',async()=>{
    const {spy,finishAll}=animations();
    // The real <Transition>, not the test stub: its leave is what used to keep the capsule a frame too long.
    const {store,wrapper}=setup({open:true,phase:'idle',layout:{ball:BALL,right:true}},{transition:false});await nextTick();
    store.open=false;store.ball=true;await nextTick();await flushPromises();
    expect(wrapper.get('.stage').classes()).toContain('morph-collapse');
    const seen:boolean[]=[];
    const observer=new MutationObserver(()=>{ if(!wrapper.get('.stage').classes().includes('morph-collapse')) seen.push(wrapper.find('[data-testid="assistant"]').exists()); });
    observer.observe(wrapper.get('.stage').element,{attributes:true,attributeFilter:['class']});
    finishAll();await flushPromises();await nextTick();await new Promise(resolve=>setTimeout(resolve,50));
    observer.disconnect();
    expect(seen.length).toBeGreaterThan(0);
    expect(seen.every(capsuleStillThere=>!capsuleStillThere)).toBe(true);
    spy.mockRestore();
  });
  it('turns around halfway instead of starting over',async()=>{
    const {runs,spy}=animations();
    const {store,wrapper}=folded();await nextTick();
    store.ball=false;store.open=true;await nextTick();await nextTick();await flushPromises();
    const running=[...runs];expect(running.length).toBeGreaterThan(0);
    store.open=false;store.ball=true;await nextTick();
    expect(running.every(run=>run.reverse.mock.calls.length===1)).toBe(true);
    expect(wrapper.get('.stage').classes()).toContain('morph-collapse');
    spy.mockRestore();
  });
  it('fades the ball out before the host moves the window under it, then back in',async()=>{
    vi.useFakeTimers();
    const {store,wrapper}=folded();await nextTick();
    store.reanchoring=7;await nextTick();
    expect(wrapper.get('.stage').classes()).toEqual(expect.arrayContaining(['moving-ball','ball-fade']));
    await vi.advanceTimersByTimeAsync(100);expect(store.reanchorReady).toHaveBeenCalledWith(7);
    store.layout={ball:{x:0,y:520,size:72},right:false};store.reanchoring=0;await nextTick();
    await vi.advanceTimersByTimeAsync(50);
    expect(wrapper.get('.stage').classes()).not.toContain('moving-ball');
    await vi.advanceTimersByTimeAsync(150);
    expect(wrapper.get('.stage').classes()).not.toContain('ball-fade');
    expect(wrapper.get('[data-testid="assistant-ball-stage"]').attributes('style')).toContain('left: 0px');
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
