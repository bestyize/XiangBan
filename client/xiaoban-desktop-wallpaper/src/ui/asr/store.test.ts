// @vitest-environment happy-dom
import { flushPromises } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import { createAsrStore, type AsrDeps } from './store.ts';
function deferred<T>() { let resolve!: (value: T) => void; const promise = new Promise<T>(r => { resolve = r; }); return { promise, resolve }; }
function setup(options: { enabled?: boolean; modelReady?: boolean; prepare?: () => Promise<unknown>; transcribe?: () => Promise<string>; deny?: boolean } = {}) {
  let enabled = options.enabled ?? false;
  let frame: ((frame: Float32Array) => void) | undefined;
  let progress: ((payload: unknown) => void) | undefined;
  let hostState: ((payload: unknown) => void) | undefined;
  const stop = vi.fn(), off = vi.fn(), notify = vi.fn();
  const invoke = vi.fn(async (command: string, args?: Record<string, unknown>) => {
    if (command === 'asr_get_status') return { enabled, modelReady: options.modelReady ?? false, engineReady: false };
    if (command === 'asr_set_enabled') { enabled = args!.enabled as boolean; return { enabled, modelReady: true, engineReady: false }; }
    if (command === 'asr_prepare') return options.prepare ? options.prepare() : { enabled, modelReady: true, engineReady: true };
    if (command === 'asr_transcribe') return options.transcribe ? options.transcribe() : '你好，小伴。';
    throw new Error(command);
  });
  const capture = vi.fn(async (onFrame: (frame: Float32Array) => void) => { if (options.deny) throw new Error('请允许麦克风'); frame = onFrame; return { stop }; });
  const store = createAsrStore({ desktop: true, invoke: invoke as AsrDeps['invoke'], notify, capture,
    listen: async (event, callback) => { if (event === 'asr-state') hostState = callback as (payload: unknown) => void; else progress = callback as (payload: unknown) => void; return off; } });
  function phrase() { for (let i = 0; i < 3; i++) frame?.(new Float32Array(1600).fill(0.1)); for (let i = 0; i < 9; i++) frame?.(new Float32Array(1600)); }
  return { store, invoke, capture, notify, stop, off, phrase, feed: (audio: Float32Array) => frame?.(audio), progress: (payload: unknown) => progress?.(payload), hostState: (payload: unknown) => hostState?.(payload) };
}
describe('local ASR lifecycle', () => {
  it('previews speech during playback, then returns the full utterance with its original echo reference', async () => {
    const h = setup({enabled:true}), text = vi.fn(), preview = vi.fn(); let playback = '正在朗读的内容';
    await h.store.activate(text, {noiseGuard:true, playbackText:()=>playback, onPreview:preview});
    const voice = Float32Array.from({length:1600}, (_, i)=>.08*Math.sin(2*Math.PI*220*i/16000));
    for(let i=0;i<8;i++) h.feed(voice);
    await flushPromises();
    expect(preview).toHaveBeenCalledWith('你好，小伴。',{playbackText:'正在朗读的内容'});
    expect(text).not.toHaveBeenCalled(); expect(h.store.listening).toBe(true);
    playback = '';
    for(let i=0;i<9;i++)h.feed(new Float32Array(1600));
    await flushPromises();
    expect(text).toHaveBeenCalledExactlyOnceWith('你好，小伴。',{playbackText:'正在朗读的内容'});
    h.store.dispose();
  });
  it('keeps preview work bounded and discards late previews after stopping', async () => {
    const pending=deferred<string>(), h=setup({enabled:true,transcribe:()=>pending.promise}), preview=vi.fn(), text=vi.fn();
    await h.store.activate(text,{playbackText:()=> 'readout',onPreview:preview});
    for(let i=0;i<30;i++) h.feed(new Float32Array(1600).fill(.1));
    expect(h.invoke.mock.calls.filter(([c])=>c==='asr_transcribe')).toHaveLength(1);
    h.store.deactivate(); pending.resolve('stale'); await flushPromises();
    expect(preview).not.toHaveBeenCalled();expect(text).not.toHaveBeenCalled();h.store.dispose();
  });
  it('does not attach an old playback reference to later speech after a silent readout',async()=>{
    const h=setup({enabled:true}), text=vi.fn();let playback='silent readout';
    await h.store.activate(text,{playbackText:()=>playback,onPreview:vi.fn()});
    for(let i=0;i<12;i++)h.feed(new Float32Array(1600));
    playback='';h.phrase();await flushPromises();expect(text).toHaveBeenCalledExactlyOnceWith('你好，小伴。');h.store.dispose();
  });
  it('follows the saved switch and engine readiness from other windows without opening a microphone', async () => {
    const h = setup({ modelReady: false });
    await h.store.init();
    expect(h.store.engineReady).toBe(false);
    h.hostState({ enabled: true, modelReady: true, engineReady: false, warming: true });
    expect(h.store.warming).toBe(true); expect(h.store.modelReady).toBe(true); expect(h.store.enabled).toBe(true); expect(h.capture).not.toHaveBeenCalled();
    h.hostState({ enabled: false, modelReady: true, engineReady: false, warming: false });
    expect(h.store.enabled).toBe(false); expect(h.store.engineReady).toBe(false); expect(h.store.warming).toBe(false);
    expect(h.off).not.toHaveBeenCalled();
    h.store.dispose();
    expect(h.off).toHaveBeenCalledTimes(2);
  });
  it('defaults off and never requests the microphone from settings', async () => {
    const h = setup(); await h.store.init(); expect(h.store.enabled).toBe(false);
    await h.store.setEnabled(true); expect(h.store.engineReady).toBe(true); expect(h.capture).not.toHaveBeenCalled();
    await h.store.setEnabled(false); expect(h.invoke.mock.calls.map(([c]) => c)).not.toContain('asr_delete_model');
    expect(h.store.enabled).toBe(false); h.store.dispose(); expect(h.off).toHaveBeenCalled();
  });
  it('loads cached files, transcribes a phrase, and stops recording on leaving chat', async () => {
    const h = setup({ enabled: true, modelReady: true }); const text = vi.fn();
    h.store.activate(text); await flushPromises(); expect(h.store.listening).toBe(true);
    h.phrase(); await flushPromises(); expect(text).toHaveBeenCalledWith('你好，小伴。');
    expect(h.notify).not.toHaveBeenCalled(); h.store.deactivate(); expect(h.stop).toHaveBeenCalledOnce(); h.store.dispose();
  });
  it('flushes and transcribes accumulated speech when user finishes recording explicitly', async () => {
    let frameCallback: ((frame: Float32Array) => void) | undefined;
    const h = setup({ enabled: true, modelReady: true });
    h.capture.mockImplementation(async cb => { frameCallback = cb; return { stop: h.stop }; });
    const text = vi.fn();
    await h.store.activate(text);
    expect(h.store.listening).toBe(true);
    // User speaks 3 frames without a following 0.9s pause
    for (let i = 0; i < 3; i++) frameCallback!(new Float32Array(1600).fill(0.1));
    expect(text).not.toHaveBeenCalled();
    // User clicks the microphone button to stop recording
    await h.store.finish();
    expect(text).toHaveBeenCalledWith('你好，小伴。');
    expect(h.store.listening).toBe(false);
    expect(h.store.recordingRequested).toBe(false);
    expect(h.stop).toHaveBeenCalledOnce();
    h.store.dispose();
  });
  it('waits for an active transcription before finishing', async () => {
    const pending = deferred<string>();
    const h = setup({ enabled: true, modelReady: true, transcribe: () => pending.promise });
    const text = vi.fn();
    await h.store.activate(text);
    h.phrase(); await flushPromises();
    let finished = false;
    const finish = h.store.finish().then(() => { finished = true; });
    await flushPromises();
    expect(finished).toBe(false);
    pending.resolve('最后一句话'); await finish;
    expect(text).toHaveBeenCalledWith('最后一句话');
    expect(finished).toBe(true);
    h.store.dispose();
  });
  it('shows download progress and ignores a completion after turning off', async () => {
    const pending = deferred<unknown>(); const h = setup({ enabled: true, prepare: () => pending.promise });
    const text = vi.fn(); h.store.activate(text); await flushPromises();
    const request = h.invoke.mock.calls.find(([command]) => command === 'asr_prepare')![1]!.requestId;
    h.progress({ requestId: request, phase: 'downloading', downloaded: 123, total: 1000 }); expect(h.store.downloaded).toBe(123);
    h.progress({ requestId: 'stale', phase: 'downloading', downloaded: 999, total: 1000 }); expect(h.store.downloaded).toBe(123);
    await h.store.setEnabled(false); pending.resolve({ enabled: true, modelReady: true, engineReady: true }); await flushPromises();
    expect(h.store.enabled).toBe(false); expect(h.capture).not.toHaveBeenCalled(); expect(text).not.toHaveBeenCalled(); h.store.dispose();
  });
  it('does not insert stale text into a different conversation', async () => {
    const pending = deferred<string>(); const h = setup({ enabled: true, transcribe: () => pending.promise });
    const before = vi.fn(), after = vi.fn(); h.store.activate(before); await flushPromises(); h.phrase(); await flushPromises();
    h.store.deactivate(); h.store.activate(after); await flushPromises(); pending.resolve('旧会话的话'); await flushPromises();
    expect(before).not.toHaveBeenCalled(); expect(after).not.toHaveBeenCalled(); h.store.dispose();
  });
  it('keeps microphone permission diagnostics out of notifications and permits retry', async () => {
    const h = setup({ enabled: true, deny: true }); h.store.activate(vi.fn()); await flushPromises();
    expect(h.store.listening).toBe(false); expect(h.store.recordingRequested).toBe(false); expect(h.store.microphoneError).toContain('麦克风');
    expect(h.notify).not.toHaveBeenCalledWith(expect.stringContaining('麦克风'), 'error');
    expect(h.invoke).toHaveBeenCalledWith('asr_report_microphone', { error: '请允许麦克风' });
    await h.store.retry(); expect(h.capture).toHaveBeenCalledTimes(2); h.store.dispose();
  });
  it('reports a model download failure without opening the microphone', async () => {
    const h = setup({ enabled: true, prepare: async () => { throw new Error('魔搭下载中断'); } });
    h.store.activate(vi.fn()); await flushPromises(); expect(h.store.error).toBe('魔搭下载中断');
    expect(h.store.preparing).toBe(false); expect(h.capture).not.toHaveBeenCalled(); h.store.dispose();
  });
  it('closes a late microphone grant without affecting a subsequent recording', async () => {
    const h = setup({ enabled: true });
    const lateStop = vi.fn(); const grant = deferred<{ stop: typeof lateStop }>();
    h.capture.mockImplementationOnce(() => grant.promise);
    const pending = h.store.activate(vi.fn()); await flushPromises();
    h.store.deactivate(); await h.store.activate(vi.fn()); expect(h.store.listening).toBe(true);
    grant.resolve({ stop: lateStop }); await pending;
    expect(lateStop).toHaveBeenCalledOnce(); expect(h.store.listening).toBe(true);
    expect(h.store.recordingRequested).toBe(true); expect(h.stop).not.toHaveBeenCalled(); h.store.dispose();
  });
});


describe('cross-window ASR settings',()=>{
  it('receives microphone diagnostics from the assistant and clears them after successful capture', async () => {
    const h = setup({ enabled: true, modelReady: true });
    await h.store.init();
    h.hostState({ enabled: true, modelReady: true, engineReady: true, microphoneError: '找不到可用麦克风' });
    expect(h.store.microphoneError).toBe('找不到可用麦克风');
    expect(h.notify).not.toHaveBeenCalled();
    await h.store.activate(vi.fn());
    expect(h.store.microphoneError).toBe('');
    expect(h.invoke).toHaveBeenCalledWith('asr_report_microphone', { error: '' });
    h.store.dispose();
  });
  it('releases recording and ignores in-flight recognition when another window disables voice',async()=>{
    const pending=deferred<string>();const h=setup({enabled:true,transcribe:()=>pending.promise});const text=vi.fn();
    await h.store.activate(text);h.phrase();await flushPromises();expect(h.store.recognizing).toBe(true);
    h.hostState({enabled:false,modelReady:true,engineReady:false,warming:false});
    expect(h.store.enabled).toBe(false);expect(h.store.recordingRequested).toBe(false);expect(h.store.listening).toBe(false);expect(h.stop).toHaveBeenCalledOnce();
    pending.resolve('旧语音');await flushPromises();expect(text).not.toHaveBeenCalled();h.store.dispose();
  });
  it('does not let an old prepare completion re-enable voice after an external switch-off',async()=>{
    const pending=deferred<unknown>();const h=setup({enabled:true,prepare:()=>pending.promise});h.store.activate(vi.fn());await flushPromises();
    h.hostState({enabled:false,modelReady:true,engineReady:false,warming:false});expect(h.store.preparing).toBe(false);
    pending.resolve({enabled:true,modelReady:true,engineReady:true});await flushPromises();expect(h.store.enabled).toBe(false);expect(h.store.engineReady).toBe(false);expect(h.capture).not.toHaveBeenCalled();h.store.dispose();
  });
  it('does not overwrite a newer event with a stale startup snapshot',async()=>{
    const pending=deferred<unknown>();const h=setup();h.invoke.mockImplementationOnce(()=>pending.promise as never);
    const init=h.store.init();await flushPromises();h.hostState({enabled:true,modelReady:true,engineReady:true});
    pending.resolve({enabled:false,modelReady:false,engineReady:false});await init;
    expect(h.store.enabled).toBe(true);expect(h.store.engineReady).toBe(true);expect(h.capture).not.toHaveBeenCalled();h.store.dispose();
  });
});

describe('recognising ahead of the end-of-turn decision', () => {
  const FRAME = 640;
  const speech = () => new Float32Array(FRAME).fill(0.2);
  const quiet = () => new Float32Array(FRAME);
  function rig(transcribe: () => string = () => '帮我查一下天气') {
    let feed!: (frame: Float32Array) => void, answer!: (probability: number) => void;
    let pending: number[] = [];
    const vad = { reset: vi.fn(), process: vi.fn(async (samples: Float32Array) => {
      pending.push(...samples); const out: number[] = [];
      while (pending.length >= 512) { const w = pending.splice(0, 512); out.push(Math.abs(w[0]) > 0.05 ? 0.9 : 0.05); }
      return out;
    }) };
    const turn = { predict: vi.fn(() => new Promise<number>(resolve => { answer = resolve; })) };
    const invoke = vi.fn(async (command: string) => {
      if (command === 'asr_get_status') return { enabled: true, modelReady: true, engineReady: true };
      if (command === 'asr_transcribe') return transcribe();
      throw new Error(command);
    });
    const store = createAsrStore({ desktop: true, invoke: invoke as AsrDeps['invoke'], notify: vi.fn(), listen: async () => () => {},
      capture: async cb => { feed = cb; return { stop: vi.fn() }; }, turnModels: async () => ({ vad, turn }) });
    const transcriptions = () => invoke.mock.calls.filter(([command]) => command === 'asr_transcribe').length;
    return { store, feed: (frame: Float32Array) => feed(frame), answer: (p: number) => answer(p), transcriptions, turn };
  }
  async function pause(h: ReturnType<typeof rig>) {
    for (let i = 0; i < 25; i++) h.feed(speech());
    await flushPromises();
    // Frames arrive 40 ms apart, so the model has answered about the previous one by the time the next comes.
    for (let i = 0; i < 6; i++) { h.feed(quiet()); await flushPromises(); }
  }

  it('recognises the utterance during the pause and reuses it when the speaker has finished', async () => {
    const h = rig(), text = vi.fn();
    await h.store.activate(text); await flushPromises();
    await pause(h);
    expect(h.transcriptions()).toBe(1); expect(text).not.toHaveBeenCalled();
    h.answer(0.9); await flushPromises();
    h.feed(quiet()); await flushPromises();
    expect(h.transcriptions()).toBe(1);
    expect(text).toHaveBeenCalledExactlyOnceWith('帮我查一下天气', { playbackText: '', settled: true });
    h.store.dispose();
  });
  it('recognises again when more was said after the pause', async () => {
    let n = 0; const h = rig(() => (++n === 1 ? '帮我查一下' : '帮我查一下天气')), text = vi.fn();
    await h.store.activate(text); await flushPromises();
    await pause(h);
    expect(h.transcriptions()).toBe(1);
    h.answer(0.1); await flushPromises(); // not finished
    for (let i = 0; i < 10; i++) h.feed(speech());
    await flushPromises();
    for (let i = 0; i < 6; i++) { h.feed(quiet()); await flushPromises(); }
    h.answer(0.9); await flushPromises();
    h.feed(quiet()); await flushPromises();
    expect(text).toHaveBeenCalledExactlyOnceWith('帮我查一下天气', { playbackText: '', settled: true });
    h.store.dispose();
  });
  it('recognises the final utterance itself when the guess failed', async () => {
    let n = 0; const h = rig(() => { if (++n === 1) throw new Error('busy'); return '帮我查一下天气'; }), text = vi.fn();
    await h.store.activate(text); await flushPromises();
    await pause(h);
    h.answer(0.9); await flushPromises();
    h.feed(quiet()); await flushPromises();
    expect(h.transcriptions()).toBe(2);
    expect(text).toHaveBeenCalledExactlyOnceWith('帮我查一下天气', { playbackText: '', settled: true });
    expect(h.store.error).toBe('');
    h.store.dispose();
  });
  it('does not guess while the assistant is talking', async () => {
    const h = rig();
    await h.store.activate(vi.fn(), { playbackText: () => '正在朗读' }); await flushPromises();
    await pause(h);
    expect(h.transcriptions()).toBe(0);
    h.store.dispose();
  });
});

describe('stray recognition results', () => {
  it('drops a stray "The" instead of passing it on, and still delivers real speech', async () => {
    let answer = 'The.';
    const h = setup({ enabled: true, transcribe: async () => answer }), text = vi.fn();
    await h.store.activate(text);
    h.phrase(); await flushPromises();
    expect(text).not.toHaveBeenCalled();
    answer = '帮我查一下天气。';
    h.phrase(); await flushPromises();
    expect(text).toHaveBeenCalledExactlyOnceWith('帮我查一下天气。');
    h.store.dispose();
  });
});

describe('sharing the microphone with another window', () => {
  it('claims the voice when the microphone is about to be used', async () => {
    const claimVoice = vi.fn();
    const invoke = vi.fn(async (command: string) => ({ asr_get_status: { enabled: true, modelReady: true, engineReady: true } } as Record<string, unknown>)[command]);
    const store = createAsrStore({ desktop: true, invoke: invoke as AsrDeps['invoke'], notify: vi.fn(), listen: async () => () => {}, claimVoice, capture: async () => ({ stop: vi.fn() }) });
    expect(claimVoice).not.toHaveBeenCalled();
    await store.activate(vi.fn());
    expect(claimVoice).toHaveBeenCalledOnce();
    store.dispose();
  });
});
