import { desktop, invoke } from '../bridge.ts';

export const fortunes = [
  { title: '花开有时', verse: '春风自有花期，心事终有回响。', blessing: '慢一点也无妨，属于你的花期正在路上。', action: '把一件惦记的小事做完。' },
  { title: '顺风而行', verse: '长风轻拂衣袖，好事正在途中。', blessing: '愿今天的努力，恰好遇上一个好机会。', action: '给想尝试的事情一个开始。' },
  { title: '平安喜乐', verse: '一盏灯火可亲，一程岁月安稳。', blessing: '愿你出入平安，日子有暖，心里有光。', action: '吃一顿热饭，照顾好自己。' },
  { title: '云开见月', verse: '浮云终会散去，月色仍在枝头。', blessing: '愿难解的心事，慢慢找到出口。', action: '暂时放下烦恼，出去走一走。' },
  { title: '良缘相逢', verse: '山海皆有归处，知音自会相逢。', blessing: '愿有人听懂你的话，也珍惜你的心意。', action: '向一个惦念的人问声好。' },
  { title: '自在如风', verse: '且听檐下风声，且看眼前山色。', blessing: '愿你有不赶路的片刻，也有随心而行的自由。', action: '留十分钟，做一件只为自己开心的事。' },
  { title: '步步生光', verse: '微光汇成星河，小步亦能千里。', blessing: '愿每一个认真生活的日子，都算数。', action: '记录今天完成的一件小事。' },
  { title: '好事将至', verse: '枝上新芽初醒，窗前喜鹊轻鸣。', blessing: '愿平常的一天，也有意料之外的欢喜。', action: '留意身边一个值得感谢的小瞬间。' },
  { title: '心有所安', verse: '灯暖一方小院，心安一处人间。', blessing: '愿你在忙碌里，也有属于自己的安稳角落。', action: '整理桌面的一小块地方。' },
  { title: '丰盈有余', verse: '细水长流不息，四时各有收成。', blessing: '愿你的付出有回音，生活有余地。', action: '给明天留一点时间，也给自己留一点力气。' },
  { title: '万物向新', verse: '旧页随风翻过，新枝迎光而生。', blessing: '愿你带着轻一点的心，迎接新的可能。', action: '试着换一种方式，完成一件熟悉的事。' },
  { title: '诸事顺遂', verse: '所行皆有风景，所盼皆有回声。', blessing: '愿所念有回应，所行有收获。', action: '选一件最重要的事，先走一步。' },
] as const;

export const wishThemes = ['平安', '顺遂', '相逢', '自在'] as const;
export interface Wish { id: string; text: string; theme: string; x: number; y: number }
export interface Journal { version: 1; saved: number[]; wishes: Wish[] }
export const emptyJournal = (): Journal => ({ version: 1, saved: [], wishes: [] });
export function parseJournal(value: unknown): Journal {
  if (!value || typeof value !== 'object') throw new Error('心愿记录格式不正确，原记录已保留');
  const data = value as Partial<Journal>;
  if (data.version !== 1 || !Array.isArray(data.saved) || !Array.isArray(data.wishes) || data.saved.length > fortunes.length || data.wishes.length > 24
    || data.saved.some(id => !Number.isInteger(id) || id < 0 || id >= fortunes.length)
    || data.wishes.some(wish => !wish || typeof wish.id !== 'string' || wish.id.length > 64 || typeof wish.text !== 'string' || !wish.text.trim() || wish.text.length > 60
      || !wishThemes.includes(wish.theme as typeof wishThemes[number]) || !Number.isFinite(wish.x) || !Number.isFinite(wish.y) || wish.x < .1 || wish.x > .9 || wish.y < .25 || wish.y > .85)) {
    throw new Error('心愿记录格式不正确，原记录已保留');
  }
  return { version: 1, saved: [...new Set(data.saved)], wishes: data.wishes };
}
export async function loadJournal(): Promise<Journal> {
  if (desktop) return parseJournal(await invoke('read_ritual_journal'));
  const stored = localStorage.getItem('xiaoban.rituals.v1');
  return stored ? parseJournal(JSON.parse(stored)) : emptyJournal();
}
export async function saveJournal(journal: Journal): Promise<void> {
  const data = parseJournal(journal);
  if (desktop) await invoke('write_ritual_journal', { data });
  else localStorage.setItem('xiaoban.rituals.v1', JSON.stringify(data));
}
export async function openRitual(scene: 'fortune' | 'pond', fortuneId?: number): Promise<void> {
  if (desktop) await invoke('open_ritual_space', { scene, fortuneId: fortuneId ?? null });
  else {
    const tab = window.open(`/?scene=ritual&ritual=${scene}${fortuneId === undefined ? '' : `&sign=${fortuneId}`}`, '_blank');
    if (!tab) throw new Error('新窗口被浏览器拦截，请允许打开预览');
    tab.opener = null;
  }
}
