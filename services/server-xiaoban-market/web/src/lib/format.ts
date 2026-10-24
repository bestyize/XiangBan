import type { Kind, Plural } from './types';

export const PLURAL: Record<Kind, Plural> = { skill: 'skills', expert: 'experts', connector: 'connectors' };
export const SINGULAR: Record<Plural, Kind> = { skills: 'skill', experts: 'expert', connectors: 'connector' };
export const KIND_NAME: Record<Kind, string> = { skill: '技能', expert: '专家', connector: '连接器' };

/** 10000 以上用「w」：109.4w。 */
export function formatCount(n: number): string {
  if (n < 10000) return String(n);
  const w = n / 10000;
  return `${w >= 100 ? Math.round(w) : w.toFixed(1).replace(/\.0$/, '')}w`;
}

export function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(bytes < 10240 ? 1 : 0)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

const pad = (n: number) => String(n).padStart(2, '0');

/** 今天 09:12 / 昨天 / 3 天前 / 2026-10-01。 */
export function formatTime(iso: string | null | undefined, now = new Date()): string {
  if (!iso) return '—';
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return '—';
  const startOfDay = (x: Date) => new Date(x.getFullYear(), x.getMonth(), x.getDate()).getTime();
  const days = Math.round((startOfDay(now) - startOfDay(d)) / 86400000);
  if (days === 0) return `今天 ${pad(d.getHours())}:${pad(d.getMinutes())}`;
  if (days === 1) return '昨天';
  if (days > 1 && days < 7) return `${days} 天前`;
  if (days >= 7 && days < 30) return `${Math.floor(days / 7)} 周前`;
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

export function formatDuration(from: string | null, to: string | null): string {
  if (!from) return '—';
  const ms = (to ? new Date(to).getTime() : Date.now()) - new Date(from).getTime();
  const s = Math.max(0, Math.round(ms / 1000));
  if (s < 60) return `${s} 秒`;
  return `${Math.floor(s / 60)} 分 ${pad(s % 60)} 秒`;
}

export function shortHash(h: string, n = 8): string {
  return h.length > n * 2 ? `${h.slice(0, n)}…${h.slice(-4)}` : h;
}

const TINTS: [string, string][] = [
  ['#F8EBDD', '#7A3E14'], ['#E3EDF8', '#24507F'], ['#E6F1E3', '#34663A'],
  ['#EFE6F5', '#613A7C'], ['#F7F0D6', '#6B5410'], ['#E1EFEF', '#235E60'],
];

/** 按标识稳定地选一组头像配色（文字与底色对比度均 ≥ 4.5:1）。 */
export function tintFor(seed: string): { bg: string; ink: string } {
  let h = 0;
  for (const c of seed) h = (h * 31 + c.codePointAt(0)!) >>> 0;
  const [bg, ink] = TINTS[h % TINTS.length];
  return { bg, ink };
}

export function initialOf(name: string): string {
  return Array.from(name.trim())[0]?.toUpperCase() ?? '?';
}

export interface Segment {
  text: string;
  hit: boolean;
}

/** 把关键词命中的部分切出来，用于高亮；不使用 v-html，避免注入。 */
export function highlight(text: string, keyword: string): Segment[] {
  const terms = keyword.split(/\s+/).map((t) => t.trim().toLowerCase()).filter(Boolean).slice(0, 5);
  if (!terms.length || !text) return [{ text, hit: false }];
  const lower = text.toLowerCase();
  const marks = new Array<boolean>(text.length).fill(false);
  for (const t of terms) {
    let i = lower.indexOf(t);
    while (i !== -1) {
      for (let k = i; k < i + t.length; k++) marks[k] = true;
      i = lower.indexOf(t, i + t.length);
    }
  }
  const out: Segment[] = [];
  let start = 0;
  for (let i = 1; i <= text.length; i++) {
    if (i === text.length || marks[i] !== marks[start]) {
      out.push({ text: text.slice(start, i), hit: marks[start] });
      start = i;
    }
  }
  return out;
}

/** 深链：由小伴桌面注册 xiaoban:// 协议后接管；客户端尚未实现时点击不会有反应。 */
export function deepLink(action: 'open' | 'install' | 'connect', kind: Kind, ns: string, slug: string, version?: string | null, sha256?: string | null): string {
  const q = new URLSearchParams();
  if (version) q.set('version', version);
  if (sha256) q.set('sha256', sha256);
  const s = q.toString();
  return `xiaoban://market/${action}/${PLURAL[kind]}/${encodeURIComponent(ns)}/${encodeURIComponent(slug)}${s ? `?${s}` : ''}`;
}
