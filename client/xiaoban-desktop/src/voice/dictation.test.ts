import { describe, expect, it, vi } from 'vitest';
import { createDictation, type InjectRequest } from './dictation.ts';

function setup(options: { inject?: (r: InjectRequest) => Promise<{ ok: boolean; error?: string }> } = {}) {
  let receiver: ((text: string) => void) | undefined;
  let partial: ((text: string) => void) | undefined;
  const asr = {
    enabled: true, microphoneSupported: true, recordingRequested: false, error: '',
    init: vi.fn(async () => {}), prepare: vi.fn(async () => true),
    activate: vi.fn(async (onText: (t: string) => void, opts?: { onPartial?(t: string): void }) => { receiver = onText; partial = opts?.onPartial; asr.recordingRequested = true; }),
    finish: vi.fn(async () => { asr.recordingRequested = false; }), deactivate: vi.fn(),
  };
  const sent: InjectRequest[] = [];
  const notify = vi.fn();
  const timers: Array<{ run: () => void; ms: number; id: number }> = [];
  const dictation = createDictation({
    asr: asr as never,
    inject: options.inject ?? (async request => { sent.push(request); return { ok: true }; }),
    notify,
    idleMs: 1000,
    setTimer: (run, ms) => { const id = timers.length + 1; timers.push({ run, ms, id }); return id; },
    clearTimer: id => { const at = timers.findIndex(t => t.id === id); if (at >= 0) timers.splice(at, 1); },
  });
  const say = async (text: string) => { receiver?.(text); await new Promise(resolve => setTimeout(resolve, 0)); };
  return { dictation, asr, sent, notify, say, partial: (t: string) => partial?.(t), timers };
}

describe('dictation', () => {
  it('puts each utterance into the target app without pressing Enter', async () => {
    const h = setup();
    expect(await h.dictation.start('Claude')).toBe(true);
    await h.say('帮我看看这个报错');
    await h.say('是什么意思');
    expect(h.sent).toEqual([
      { target: 'Claude', text: '帮我看看这个报错', press_enter: false },
      { target: 'Claude', text: '是什么意思', press_enter: false },
    ]);
    expect(h.dictation.last).toBe('是什么意思');
  });

  it('presses Enter only when the user says send, and ends on the stop phrase', async () => {
    const h = setup();
    await h.dictation.start('Claude');
    await h.say('修一下登录页');
    await h.say('发送。');
    expect(h.sent[1]).toEqual({ target: 'Claude', text: '', press_enter: true });
    await h.say('结束听写');
    expect(h.dictation.active).toBe(false);
    expect(h.asr.finish).toHaveBeenCalled();
    expect(h.sent).toHaveLength(2);
  });

  it('separates two Latin utterances with a space but leaves Chinese alone', async () => {
    const h = setup();
    await h.dictation.start();
    await h.say('use');
    await h.say('React');
    await h.say('组件');
    expect(h.sent.map(s => s.text)).toEqual(['use', ' React', '组件']);
  });

  it('ignores noise and fillers', async () => {
    const h = setup();
    await h.dictation.start();
    await h.say('嗯');
    await h.say('。');
    await h.say('(noise)');
    expect(h.sent).toEqual([]);
  });

  it('keeps utterances in order even when pasting is slow', async () => {
    const order: string[] = [];
    let release!: () => void;
    const h = setup({ inject: async request => { if (request.text === '第一句') await new Promise<void>(resolve => { release = resolve; }); order.push(request.text); return { ok: true }; } });
    await h.dictation.start();
    await h.say('第一句');
    await h.say('第二句');
    expect(order).toEqual([]);
    release();
    await new Promise(resolve => setTimeout(resolve, 0)); await new Promise(resolve => setTimeout(resolve, 0));
    expect(order).toEqual(['第一句', '第二句']);
  });

  it('reports a failure once and stops after the second in a row', async () => {
    const h = setup({ inject: async () => ({ ok: false, error: '未找到标题包含「Claude」的可视窗口' }) });
    await h.dictation.start();
    await h.say('你好');
    expect(h.dictation.active).toBe(true);
    await h.say('在吗');
    await new Promise(resolve => setTimeout(resolve, 0));
    expect(h.dictation.active).toBe(false);
    expect(h.notify).toHaveBeenCalledWith('未找到标题包含「Claude」的可视窗口');
    expect(h.notify).toHaveBeenCalledWith('连续两次没能送到 Claude，已结束听写');
  });

  it('does not start when voice recognition is off or not ready', async () => {
    const off = setup(); off.asr.enabled = false;
    expect(await off.dictation.start()).toBe(false);
    expect(off.notify).toHaveBeenCalledWith('请先在设置 → 声音里开启语音识别');
    const notReady = setup(); notReady.asr.prepare = vi.fn(async () => false); notReady.asr.error = '模型还在下载';
    expect(await notReady.dictation.start()).toBe(false);
    expect(notReady.notify).toHaveBeenCalledWith('模型还在下载');
    expect(notReady.dictation.active).toBe(false);
  });

  it('stops by itself after a long silence and when the microphone is taken away', async () => {
    const h = setup();
    await h.dictation.start();
    h.timers[0].run();
    await new Promise(resolve => setTimeout(resolve, 0));
    expect(h.dictation.active).toBe(false);
    expect(h.notify).toHaveBeenCalledWith('很久没说话，已结束听写');

    const taken = setup();
    await taken.dictation.start();
    taken.dictation.micLost();
    await taken.say('之后说的话');
    expect(taken.dictation.active).toBe(false);
    expect(taken.sent).toEqual([]);
  });

  it('shows what is being said and clears it once final', async () => {
    const h = setup();
    await h.dictation.start();
    h.partial('正在说');
    expect(h.dictation.partial).toBe('正在说');
    await h.say('正在说完了');
    expect(h.dictation.partial).toBe('');
  });
});
