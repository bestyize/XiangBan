import { describe, expect, it, vi } from 'vitest';
import { createSpeechStore, speechDefaults, type SpeechDeps } from './store.ts';
import { speechText } from './text.ts';
function fixture() {
  const audios: Pick<HTMLAudioElement, 'play' | 'pause' | 'src' | 'onended' | 'onerror'>[] = [];
  const invoke = vi.fn(async (command: string, args?: Record<string, unknown>) => {
    if (command === 'tts_get_settings') return { ...speechDefaults };
    if (command === 'tts_save_settings') return args?.settings;
    if (command === 'tts_list_voices') throw new Error('network unavailable');
    if (command === 'tts_synthesize' || command === 'tts_synthesize_part') return [1, 2, 3];
  });
  const revoke = vi.fn(), notify = vi.fn();
  const store = createSpeechStore({
    desktop: true, invoke: invoke as SpeechDeps['invoke'], revoke, notify, url: () => 'blob:speech',
    audio: src => { const a = { src, play: vi.fn(async () => {}), pause: vi.fn(), onended: null, onerror: null }; audios.push(a); return a; },
  });
  return { store, invoke, audios, revoke, notify };
}
describe('speech lifecycle', () => {
  it('reads link labels and prose while omitting fenced commands and destination URLs', () => {
    const plain = speechText('# 你好\n\n**小伴**，打开[这里](https://private.example/token)。\n\n```sh\nrm -rf /test\n```');
    expect(plain).toContain('小伴，打开这里');
    expect(plain).not.toContain('https://'); expect(plain).not.toContain('rm -rf'); expect(plain).not.toContain('**');
  });
  it('loads with auto-reading off and merges serialized settings without losing earlier changes', async () => {
    const h = fixture(); await h.store.init();
    expect(h.store.settings.autoRead).toBe(false);
    await Promise.all([h.store.save({ rate: 20 }), h.store.save({ volume: 75 })]);
    expect(h.store.settings).toMatchObject({ rate: 20, volume: 75, autoRead: false });
    const last = h.invoke.mock.calls.filter(c => c[0] === 'tts_save_settings').at(-1);
    expect(last?.[1]?.settings).toMatchObject({ rate: 20, volume: 75 });
  });
  it('revokes the MP3 URL on stop and discards audio that arrives after cancellation', async () => {
    const h = fixture(); await h.store.speak('你好', 'm1');
    expect(h.store.status).toBe('playing'); expect(h.audios).toHaveLength(1);
    await h.store.stop();
    expect(h.store.status).toBe('idle'); expect(h.revoke).toHaveBeenCalledWith('blob:speech');
    let resolve!: (bytes: number[]) => void;
    h.invoke.mockImplementation(async command => command === 'tts_synthesize' ? new Promise<number[]>(r => { resolve = r; }) : undefined);
    const pending = h.store.speak('迟到的声音', 'm2');
    await vi.waitFor(() => expect(resolve).toBeTypeOf('function'));
    await h.store.stop(); resolve([1, 2]); await pending;
    expect(h.audios).toHaveLength(1); expect(h.store.status).toBe('idle');
  });
  it('keeps fallback voices on a failed refresh and prevents playback after disposal', async () => {
    const h = fixture(); await h.store.loadVoices();
    expect(h.store.voices.some(v => v.shortName === 'zh-CN-XiaoxiaoNeural')).toBe(true);
    expect(h.store.error).toContain('network');
    h.store.dispose(); await h.store.speak('你好', 'm');
    expect(h.audios).toHaveLength(0);
  });
});

describe('reading a streamed answer', () => {
  it('synthesises sentences as they appear, plays them in turn and goes idle after the last', async () => {
    const { store, invoke, audios } = fixture(); await store.init();
    const onStart = vi.fn();
    const stream = store.stream!('a1', undefined, { onStart });
    stream.push('第一句。第二');
    await vi.waitFor(() => expect(audios).toHaveLength(1));
    expect(invoke).toHaveBeenCalledWith('tts_synthesize_part', { text: '第一句。', options: null });
    expect(store.status).toBe('playing'); expect(onStart).toHaveBeenCalledOnce();
    const done = stream.end('第一句。第二句。');
    (audios[0].onended as () => void)();
    await vi.waitFor(() => expect(audios).toHaveLength(2));
    (audios[1].onended as () => void)(); await done;
    expect(store.status).toBe('idle'); expect(stream.spoken).toBe('第一句。第二句。');
  });
  it('stops with the rest of reading: stop() ends the stream and cancels on the host', async () => {
    const { store, invoke, audios } = fixture(); await store.init();
    const stream = store.stream!('a1');
    stream.push('一。二。'); await vi.waitFor(() => expect(audios).toHaveLength(1));
    await store.stop();
    expect(audios[0].pause).toHaveBeenCalled(); expect(invoke).toHaveBeenCalledWith('tts_cancel');
    expect(store.status).toBe('idle'); expect(stream.active).toBe(false);
  });
});
