import { describe, expect, it } from 'vitest';
import { ACK_PHRASES, ackKind, chooseAck } from './ack.ts';

describe('ackKind', () => {
  it('acknowledges a request to do something', () => {
    for (const text of ['帮我整理一下下载文件夹', '把周报写完，然后回复张工的邮件。', '打开设置', '麻烦查一下明天的天气', '可以帮我翻译这段话吗']) expect(ackKind(text)).toBe('task');
  });
  it('acknowledges a question that needs thinking about', () => {
    for (const text of ['明天北京会下雨吗', '为什么天是蓝的', '这两个方案有什么区别', '我想知道怎么才能睡得更好']) expect(ackKind(text)).toBe('question');
  });
  it('says nothing to a greeting, thanks, goodbye or bare agreement', () => {
    for (const text of ['你好', '你好小伴', '你好呀！', '您好', '嗨', '早上好', '晚安', '在吗', '你好吗', '谢谢你', '谢谢小伴', '再见', '好的', '嗯嗯', '可以', 'hello']) expect(ackKind(text)).toBeNull();
  });
  it('says nothing to a feeling, a statement or something too short to tell', () => {
    for (const text of ['今天好累', '我有点焦虑', '今天天气不错', '嗯', '？', '']) expect(ackKind(text)).toBeNull();
  });
  it('has phrases for every kind it can return', () => {
    expect(ACK_PHRASES.task.length).toBeGreaterThan(1); expect(ACK_PHRASES.question.length).toBeGreaterThan(1);
  });
});

describe('chooseAck', () => {
  it('never repeats the phrase used last', () => {
    for (const kind of ['task', 'question'] as const) for (const last of ACK_PHRASES[kind]) for (const pick of [0, 0.5, 0.99]) expect(chooseAck(ACK_PHRASES[kind], last, () => pick)).not.toBe(last);
  });
  it('uses the only phrase there is, and says nothing when there is none', () => {
    expect(chooseAck(['好的'], '好的')).toBe('好的');
    expect(chooseAck([], undefined)).toBeUndefined();
  });
});
