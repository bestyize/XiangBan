import { describe, expect, it } from 'vitest';
import { isPlaybackEcho, isSpeechText } from './voice-guard.ts';

describe('assistant playback echo', () => {
  it('matches partial readout with punctuation, case and small ASR mistakes', () => {
    expect(isPlaybackEcho('先休息片刻', '可以先休息片刻，再继续工作。')).toBe(true);
    expect(isPlaybackEcho('床前明月光', '床边的故事：窗前明月光。')).toBe(true);
    expect(isPlaybackEcho('OPEN THE FILE', 'Then, open the file.')).toBe(true);
    expect(isPlaybackEcho('请先休息一会', '请先休息一会儿，然后再继续。')).toBe(true);
  });
  it('preserves new questions and short stop requests', () => {
    expect(isPlaybackEcho('等等，我想问明天的天气', '今天适合先去散步，然后回来休息。')).toBe(false);
    expect(isPlaybackEcho('停', '今天适合散步')).toBe(false);
    expect(isPlaybackEcho('不是，我想问另外一个问题', '我想问明天的天气')).toBe(false);
  });
  it('ignores punctuation and recognizer noise labels', () => {
    for (const text of ['……', '[noise]', '（音乐）', '<|nospeech|>', '🎵']) expect(isSpeechText(text)).toBe(false);
    for (const text of ['停', '你好', '帮我播放音乐']) expect(isSpeechText(text)).toBe(true);
  });
});
