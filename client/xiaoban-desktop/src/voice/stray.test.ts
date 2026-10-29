import { describe, expect, it } from 'vitest';
import { isStray } from './stray.ts';

describe('isStray', () => {
  it('catches the stray words the recogniser produces on noise, however they are written', () => {
    for (const text of ['The', 'The.', 'the', ' THE ', '<|en|><|NEUTRAL|>The.', 'The the', 'Yeah.', 'Uh', 'And the', 'Oh, the']) expect(isStray(text), text).toBe(true);
  });
  it('keeps anything with Chinese in it, and real English', () => {
    for (const text of ['你好', '好的', 'The 小伴', '打开 the 设置', 'Open the settings', 'hello', 'hi', 'thanks', 'OK', 'what time is it', 'The weather today']) expect(isStray(text), text).toBe(false);
  });
  it('says nothing about an empty result', () => {
    expect(isStray('')).toBe(false); expect(isStray('。')).toBe(false);
  });
});
