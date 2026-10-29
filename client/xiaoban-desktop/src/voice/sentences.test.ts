import { describe, expect, it } from 'vitest';
import { SentenceChunker } from './sentences.ts';

function feed(parts: string[]) {
  const chunker = new SentenceChunker();
  const out: string[] = [];
  let text = '';
  for (const part of parts) { text += part; out.push(...chunker.push(text)); }
  out.push(...chunker.flush());
  return out;
}

describe('SentenceChunker', () => {
  it('emits each sentence as soon as it ends, whatever the stream boundaries', () => {
    expect(feed(['好的，', '我来看看。今天', '天气不错！要出门吗', '？'])).toEqual(['好的，我来看看。', '今天天气不错！', '要出门吗？']);
  });
  it('keeps the unfinished tail until the answer is complete', () => {
    const chunker = new SentenceChunker();
    expect(chunker.push('第一句。第二句还没')).toEqual(['第一句。']);
    expect(chunker.push('第一句。第二句还没说完')).toEqual([]);
    expect(chunker.flush()).toEqual(['第二句还没说完']);
  });
  it('strips Markdown and skips code blocks entirely', () => {
    expect(feed(['## 结论\n', '**建议**先休息。\n```js\nconsole.log("不要读。");\n```\n', '然后[看文档](https://x.y)。'])).toEqual(['结论', '建议先休息。', '然后看文档。']);
  });
  it('treats list items and lines as sentences', () => {
    expect(feed(['- 小伴桌面\n- 供应商', '配置\n'])).toEqual(['小伴桌面', '供应商配置']);
  });
  it('cuts a long clause at a pause so the first words are not held back', () => {
    const long = '这是一段非常非常长的说明文字没有句号但是有逗号'.repeat(3) + '，后面还有';
    const out = feed([long]);
    expect(out[0].endsWith('，')).toBe(true);
    expect(out.join('')).toContain('后面还有');
  });
  it('splits English at a period followed by a space but not inside numbers', () => {
    expect(feed(['Version 2.1 is out. It is faster.'])).toEqual(['Version 2.1 is out.', 'It is faster.']);
  });
  it('does not read a list number on its own', () => {
    expect(feed(['1.', ' 先打开设置。'])).toEqual(['先打开设置。']);
  });
  it('cuts the first sentence at its first comma once it is long enough, so the first words come sooner', () => {
    expect(feed(['好的，我来帮你查一下今天的天气，然后告诉你结果。'])).toEqual(['好的，我来帮你查一下今天的天气，', '然后告诉你结果。']);
  });
  it('keeps later sentences whole', () => {
    expect(feed(['好的。我先看一下天气，再看一下时间，然后告诉你。'])).toEqual(['好的。', '我先看一下天气，再看一下时间，然后告诉你。']);
  });
  it('does not read emoji, and does not read a sentence made only of them', () => {
    expect(feed(['好的😊，我来看看。', '😄', '再见👋！'])).toEqual(['好的，我来看看。', '再见！']);
  });
});
