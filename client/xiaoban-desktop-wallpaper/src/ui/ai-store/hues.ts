import { BookOpen, Briefcase, Code2, Coffee, PenLine, Puzzle, TrendingUp } from 'lucide-vue-next';
import type { Component } from 'vue';

/** 设计稿 HUES：[高光, 主色, 同色投影, 顶部淡晕]；浅晕在 CSS 里用 color-mix 推导，深色模式同样可用。 */
export const HUES = {
  amber: ['#F6A94B', '#E07A1F'], blue: ['#5E9BFF', '#2F6FDB'], green: ['#4CC08A', '#23885A'], rose: ['#F27A9F', '#D63F6E'],
  purple: ['#9B83F2', '#6A4BD6'], teal: ['#3FC1C9', '#17858C'], coral: ['#FF8A6B', '#E0573A'], indigo: ['#7C8CF8', '#4A57D0'],
} as const;
export type HueName = keyof typeof HUES;
const order = Object.keys(HUES) as HueName[];

const byCategory: Record<string, HueName> = {
  开发: 'blue', tech: 'blue', 办公: 'amber', business: 'amber', 创作: 'coral', creative: 'coral',
  金融: 'green', finance: 'green', 学习: 'purple', education: 'purple', 生活: 'teal', lifestyle: 'teal',
};
const icons: Record<string, Component> = {
  开发: Code2, tech: Code2, 办公: Briefcase, business: Briefcase, 创作: PenLine, creative: PenLine,
  金融: TrendingUp, finance: TrendingUp, 学习: BookOpen, education: BookOpen, 生活: Coffee, lifestyle: Coffee,
};
function hash(text: string) { let h = 0; for (const ch of text) h = (h * 31 + ch.charCodeAt(0)) >>> 0; return h; }

/** 按分类稳定取色；无已知分类时按 key 哈希，同一条目始终同色。 */
export function hueFor(category: string | undefined, key: string): HueName {
  return (category && byCategory[category]) || order[hash(key || category || '') % order.length];
}
/** 返回内联 CSS 变量，配合 store-cards.css 的 .xc-ic / .xc 使用。 */
export function hueStyle(category: string | undefined, key: string) {
  const [hi, hue] = HUES[hueFor(category, key)];
  return { '--h-hi': hi, '--h': hue };
}
export function categoryIcon(category: string | undefined): Component { return (category && icons[category]) || Puzzle; }
