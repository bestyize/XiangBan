import { describe, expect, it } from 'vitest';
import { ACK_DELAY_MS, ACK_PHRASES, ackDelay, ackKind, chooseAck } from './ack.ts';

describe('ackKind', () => {
  it('greets back, answering the greeting that was given', () => {
    const cases: [string, string][] = [['你好', 'hello'], ['你好小伴', 'hello'], ['你好呀！', 'hello'], ['小伴你好', 'hello'], ['您好', 'hello'], ['嗨', 'hello'], ['hello', 'hello'],
      ['早上好', 'morning'], ['早', 'morning'], ['中午好', 'noon'], ['下午好', 'afternoon'], ['晚上好', 'evening'], ['在吗', 'present'], ['小伴你在吗', 'present'], ['你好吗', 'howru'],
      ['谢谢', 'thanks'], ['谢谢你', 'thanks'], ['谢谢小伴', 'thanks'], ['辛苦了', 'thanks'], ['晚安', 'goodnight'], ['再见', 'bye'], ['拜拜', 'bye']];
    for (const [text, kind] of cases) expect(ackKind(text), text).toBe(kind);
  });
  it('acknowledges a request to do something', () => {
    for (const text of ['帮我整理一下下载文件夹', '把周报写完，然后回复张工的邮件。', '打开设置', '麻烦查一下明天的天气', '可以帮我翻译这段话吗', '你好，帮我查一下天气']) expect(ackKind(text), text).toBe('task');
  });
  it('acknowledges a question that needs thinking about', () => {
    for (const text of ['明天北京会下雨吗', '为什么天是蓝的', '这两个方案有什么区别', '我想知道怎么才能睡得更好']) expect(ackKind(text), text).toBe('question');
  });
  it('says nothing to bare agreement, a feeling, a statement or something too short to tell', () => {
    for (const text of ['好的', '嗯嗯', '可以', '对', '今天好累', '我有点焦虑', '今天天气不错', '嗯', '？', '']) expect(ackKind(text), text).toBeNull();
  });
  it('has more than one phrase for the kinds that come up often, and a phrase for every kind', () => {
    for (const kind of Object.keys(ACK_PHRASES) as (keyof typeof ACK_PHRASES)[]) expect(ACK_PHRASES[kind].length).toBeGreaterThan(0);
    for (const kind of ['hello', 'task', 'question', 'thanks'] as const) expect(ACK_PHRASES[kind].length).toBeGreaterThan(1);
  });
  it('says a greeting back at once, and a request only after a pause', () => {
    expect(ackDelay('hello')).toBe(0); expect(ackDelay('thanks')).toBe(0);
    expect(ackDelay('task')).toBe(ACK_DELAY_MS); expect(ackDelay('question')).toBe(ACK_DELAY_MS);
  });
});

describe('chooseAck', () => {
  it('never repeats the phrase used last', () => {
    for (const kind of ['hello', 'task', 'question', 'thanks'] as const) for (const last of ACK_PHRASES[kind]) for (const pick of [0, 0.5, 0.99]) expect(chooseAck(ACK_PHRASES[kind], last, () => pick)).not.toBe(last);
  });
  it('uses the only phrase there is, and says nothing when there is none', () => {
    expect(chooseAck(['好的'], '好的')).toBe('好的');
    expect(chooseAck([], undefined)).toBeUndefined();
  });
});
