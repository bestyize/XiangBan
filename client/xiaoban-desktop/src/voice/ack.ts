/**
 * What the assistant says first, while the answer is still being thought out, or at once when a reply needs no
 * thinking. What is said has to fit what was heard: a greeting is greeted back, a request is taken up, a question
 * is thought about, and anything else gets nothing, since saying nothing beats saying the wrong thing.
 */
export type AckKind =
  | 'hello' | 'morning' | 'noon' | 'afternoon' | 'evening' | 'present' | 'howru' | 'thanks' | 'goodnight' | 'bye'
  | 'task' | 'question';

export const ACK_PHRASES: Record<AckKind, readonly string[]> = {
  hello: ['你好呀，我在', '嗨，我在呢'],
  morning: ['早上好呀'],
  noon: ['中午好呀'],
  afternoon: ['下午好呀'],
  evening: ['晚上好呀'],
  present: ['在的，我在'],
  howru: ['我挺好的呀'],
  thanks: ['不客气', '不用谢，随时找我'],
  goodnight: ['晚安，好梦'],
  bye: ['再见，下次再聊'],
  task: ['好的，我来看看', '好，我来处理'],
  question: ['我想想', '嗯，我想想'],
};
/** Every phrase there is, for preparing them in advance. */
export const ALL_ACK_PHRASES: readonly string[] = Object.values(ACK_PHRASES).flat();
/** Said at once: the reply needs no thinking, and a pause before it would only make it late. */
const IMMEDIATE = new Set<AckKind>(['hello', 'morning', 'noon', 'afternoon', 'evening', 'present', 'howru', 'thanks', 'goodnight', 'bye']);
/** Nothing is said before this long after the question is sent; a quick answer needs no acknowledgement. */
export const ACK_DELAY_MS = 700;
export const ackDelay = (kind: AckKind) => (IMMEDIATE.has(kind) ? 0 : ACK_DELAY_MS);

const strip = (text: string) => text.normalize('NFKC').toLowerCase().replace(/[\s\p{P}\p{S}]/gu, '');
/** The person addressed, before or after, and particles: "小伴你好呀". */
const ADDRESS = '(?:小伴|你|您|呀|啊|啦|哦|呢|哈|吧|了|的|大家|啦啦|嘿|喂)*';
const social = (words: string) => new RegExp(`^${ADDRESS}(?:${words})${ADDRESS}$`);
const GREETINGS: [AckKind, RegExp][] = [
  ['hello', social('你好|您好|嗨|哈喽|hello|hi|hey|哈啰')],
  ['morning', social('早安|早上好|早|早啊|早晨')],
  ['noon', social('中午好|午安')],
  ['afternoon', social('下午好')],
  ['evening', social('晚上好')],
  ['present', social('在吗|在不在|在么')],
  ['howru', social('你好吗|你怎么样|最近好吗|最近怎么样|你还好吗')],
  ['thanks', social('谢谢|多谢|感谢|谢啦|谢谢你|辛苦了|辛苦')],
  ['goodnight', social('晚安')],
  ['bye', social('再见|拜拜|回头见|下次聊|先这样')],
];
/** Bare agreement ("好的", "嗯"): there is nothing to answer, so nothing is said. */
const AGREEMENT = social('好的|好吧|好|行|可以|对|是的|嗯|哦|嗯嗯|哦哦|哈哈|呵呵');
const FEELING = /难过|伤心|郁闷|好烦|烦死|崩溃|焦虑|压力|好累|累死|孤独|想哭|难受|不开心|失眠|害怕|生气/;
const TASK = /^(?:请|麻烦|能不能|能否|可以|帮我|帮忙|给我|替我|把|打开|关闭|查|搜索|搜|找|写|整理|改|修改|创建|新建|删除|生成|翻译|总结|分析|设置|设定|切换|播放|发送|下载|安装|运行|读|看看|看一下|提醒)|帮我|给我|替我/;
const QUESTION = /[吗么呢]$|什么|怎么|为啥|如何|多少|是不是|有没有|会不会|哪(?:个|里|些|儿)|谁|区别|原理|意思/;

/** What fits as the first thing said, or null when nothing should be. */
export function ackKind(text: string): AckKind | null {
  const said = strip(text);
  if (!said) return null;
  for (const [kind, pattern] of GREETINGS) if (pattern.test(said)) return kind;
  if (said.length < 3 || AGREEMENT.test(said)) return null;
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
