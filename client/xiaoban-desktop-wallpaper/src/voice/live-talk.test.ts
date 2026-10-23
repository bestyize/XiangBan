import { nextTick, reactive } from 'vue';
import { flushPromises } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import type { AsrActivationOptions, TranscriptContext } from '../ui/asr/store.ts';
import { createLiveTalk, LIVE_SEND_GRACE_MS } from './live-talk.ts';

function setup() {
  let clock = 1_000_000;
  const timers: { id: number; at: number; run: () => void }[] = [];
  let nextId = 1;
  const advance = (ms: number) => {
    clock += ms;
    for (;;) {
      const due = timers.filter(timer => timer.at <= clock).sort((a, b) => a.at - b.at)[0];
      if (!due) break;
      timers.splice(timers.indexOf(due), 1);
      due.run();
    }
  };
  const chat = reactive({
    currentId: 's1' as string | null,
    views: { s1: { running: false, items: [] as any[], approvals: [] as unknown[], questions: [] as unknown[] } } as Record<string, any>,
    current() { return chat.currentId ? chat.views[chat.currentId] ?? null : null; },
    send: vi.fn(async (text: string, _attachments?: never[], _note?: string) => {
      const view = chat.views.s1; view.items.push({ kind: 'user', id: 'u' + view.items.length, text, images: [] }); view.running = true; return true;
    }),
    cancel: vi.fn(async () => { chat.views.s1.running = false; }),
  });
  let heard: (text: string, context?: TranscriptContext) => void = () => {};
  let options: AsrActivationOptions = {};
  const asr = reactive({
    enabled: true, microphoneSupported: true, listening: false, recognizing: false, speechActive: false, recordingRequested: false, error: '',
    init: vi.fn(async () => {}), prepare: vi.fn(async () => true),
    activate: vi.fn(async (callback: typeof heard, opts?: AsrActivationOptions) => { asr.recordingRequested = true; asr.listening = true; heard = callback; options = opts ?? {}; }),
    deactivate: vi.fn(() => { asr.recordingRequested = false; asr.listening = false; }),
  });
  const speech = reactive({
    status: 'idle' as 'idle' | 'synthesizing' | 'playing',
    speak: vi.fn(async () => { speech.status = 'playing'; }),
    stop: vi.fn(async () => { speech.status = 'idle'; }),
    ack: vi.fn(() => true), prepareAck: vi.fn(async () => {}),
    stream: undefined as any,
  });
  const notify = vi.fn();
  const live = createLiveTalk({
    chat: chat as never, asr: asr as never, speech: speech as never, notify, now: () => clock,
    setTimer: (run, ms) => { const timer = { id: nextId++, at: clock + ms, run }; timers.push(timer); return timer.id; },
    clearTimer: id => { const at = timers.findIndex(timer => timer.id === id); if (at >= 0) timers.splice(at, 1); },
  });
  const answer = (text: string) => { const view = chat.views.s1; view.items.push({ kind: 'assistant', id: 'a' + view.items.length, text, streaming: false }); view.running = false; };
  return { live, chat, asr, speech, notify, advance, answer, say: (text: string, context?: TranscriptContext) => heard(text, context), preview: (text: string) => options.onPreview?.(text, { playbackText: '' }) };
}

describe('实时对话 in the conversation', () => {
  it('sends after a pause, reads the answer and listens again, round after round', async () => {
    const { live, chat, asr, speech, advance, answer, say } = setup();
    await live.start();
    expect(live.active).toBe(true); expect(live.status).toBe('在听');
    say('下周一有什么会');
    expect(live.heard).toBe('下周一有什么会');
    advance(LIVE_SEND_GRACE_MS + 5); await flushPromises();
    expect(chat.send).toHaveBeenCalledWith('下周一有什么会');
    expect(live.status).toBe('在想');
    answer('上午十点有周会。');
    live.replied('s1', '上午十点有周会。', 'a1');
    expect(speech.speak).toHaveBeenCalledWith('上午十点有周会。', 'a1', undefined, true);
    expect(live.status).toBe('在说');
    await flushPromises();
    speech.status = 'idle'; await nextTick(); await flushPromises();
    expect(live.status).toBe('在听'); expect(asr.recordingRequested).toBe(true);
    say('谢谢'); advance(LIVE_SEND_GRACE_MS + 5); await flushPromises();
    expect(chat.send).toHaveBeenCalledTimes(2);
    live.dispose();
  });

  it('stops reading when the user talks over it, and tells the model how far it got', async () => {
    const { live, chat, speech, advance, answer, say, preview } = setup();
    await live.start();
    say('讲个长一点的故事'); advance(LIVE_SEND_GRACE_MS + 5); await flushPromises();
    answer('从前有座山……'); live.replied('s1', '从前有座山……');
    await flushPromises();
    speech.stop.mockClear();
    preview('等一下');
    expect(speech.stop).toHaveBeenCalled(); expect(live.status).toBe('在听');
    say('换一个'); advance(LIVE_SEND_GRACE_MS + 5); await flushPromises();
    expect(chat.send).toHaveBeenLastCalledWith('换一个', [], expect.stringContaining('打断'));
    live.dispose();
  });

  it('cancels a running answer when the user starts talking and keeps the microphone', async () => {
    const { live, chat, asr, advance, say } = setup();
    await live.start();
    say('查一下天气'); advance(LIVE_SEND_GRACE_MS + 5); await flushPromises();
    say('算了不用了');
    expect(chat.cancel).toHaveBeenCalled();
    expect(live.activity()).toBe(true);
    expect(asr.deactivate).toHaveBeenCalledTimes(1); // only the one before sending
    live.dispose();
  });

  it('mutes without ending, and 结束 stops everything but the task', async () => {
    const { live, chat, asr, speech, advance, answer, say } = setup();
    await live.start();
    await live.toggleMute();
    expect(live.status).toBe('已静音'); expect(asr.recordingRequested).toBe(false); expect(live.active).toBe(true);
    await live.toggleMute();
    expect(asr.recordingRequested).toBe(true);
    say('写个周报'); advance(LIVE_SEND_GRACE_MS + 5); await flushPromises();
    live.end();
    expect(live.active).toBe(false); expect(asr.recordingRequested).toBe(false); expect(chat.cancel).not.toHaveBeenCalled();
    answer('好了'); speech.speak.mockClear();
    live.replied('s1', '好了');
    expect(speech.speak).not.toHaveBeenCalled();
    expect(live.activity()).toBe(false);
    live.dispose();
  });

  it('does not start when voice recognition is off', async () => {
    const { live, asr, notify } = setup();
    asr.enabled = false;
    await live.start();
    expect(live.active).toBe(false); expect(notify).toHaveBeenCalledWith(expect.stringContaining('语音识别'), 'info');
    live.dispose();
  });
});
