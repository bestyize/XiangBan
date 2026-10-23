/**
 * Short sounds of "I heard you", played while the answer is still being thought out. What is said has to fit
 * what was asked, so it depends on the kind of request, and a request of no kind gets no acknowledgement:
 * a greeting answered with "好的" sounds worse than a short wait.
 */
export type AckKind = 'task' | 'question';
export const ACK_PHRASES: Record<AckKind, readonly string[]> = {
  task: ['好的，我来看看', '好，我来处理'],
  question: ['我想想', '嗯，我想想'],
};
/** Every phrase there is, for preparing them in advance. */
export const ALL_ACK_PHRASES: readonly string[] = Object.values(ACK_PHRASES).flat();
/** Nothing is said before this long after the question is sent; a quick answer needs no acknowledgement. */
export const ACK_DELAY_MS = 700;

const strip = (text: string) => text.normalize('NFKC').toLowerCase().replace(/[\s\p{P}\p{S}]/gu, '');
/** Greetings, thanks, goodbyes and bare agreement, with only the person addressed or a particle after them. */
const SOCIAL = /^(?:你好|您好|嗨|哈喽|hello|hi|hey|早安|早上好|中午好|下午好|晚上好|晚安|在吗|在不在|你好吗|谢谢|多谢|感谢|辛苦|再见|拜拜|好的|好吧|好|行|可以|对|是的|嗯|哦)(?:小伴|你|您|呀|啊|啦|哦|呢|哈|吧|了|的|大家|啦啦)*$/;
const FEELING = /难过|伤心|郁闷|好烦|烦死|崩溃|焦虑|压力|好累|累死|孤独|想哭|难受|不开心|失眠|害怕|生气/;
const TASK = /^(?:请|麻烦|能不能|能否|可以|帮我|帮忙|给我|替我|把|打开|关闭|查|搜索|搜|找|写|整理|改|修改|创建|新建|删除|生成|翻译|总结|分析|设置|设定|切换|播放|发送|下载|安装|运行|读|看看|看一下|提醒)|帮我|给我|替我/;
const QUESTION = /[吗么呢]$|什么|怎么|为啥|如何|多少|是不是|有没有|会不会|哪(?:个|里|些|儿)|谁|区别|原理|意思/;

/** What kind of acknowledgement fits, or null when nothing should be said. */
export function ackKind(text: string): AckKind | null {
  const said = strip(text);
  if (said.length < 3 || SOCIAL.test(said)) return null;
  if (TASK.test(said)) return 'task';
  if (FEELING.test(said)) return null;
  // A very short remark ("好吗") is not worth an acknowledgement.
  if (said.length >= 5 && QUESTION.test(said)) return 'question';
  return null;
}

/** One phrase, never the one used last. `pick` returns a number in [0, 1). */
export function chooseAck<T>(clips: readonly T[], last: T | undefined, pick: () => number = Math.random): T | undefined {
  const options = clips.length > 1 ? clips.filter(clip => clip !== last) : clips;
  return options.length ? options[Math.min(options.length - 1, Math.floor(pick() * options.length))] : undefined;
}
