// 小伴市场：界面用到的类型和不依赖界面的判断。
import type { SkillReview } from './catalog.ts';
import type { MarketConnector } from './connector-plugins.ts';

export type MarketKind = 'skills' | 'experts' | 'connectors';
export type Variant = 'single' | 'team' | 'bundle';

export interface MarketItem {
  connector?: MarketConnector;
  slug: string; name: string; description: string; version: string; namespace: string; kind?: string;
  iconUrl?: string | null; source?: string; origin?: string; downloads?: number; stars?: number;
  variant?: Variant; memberCount?: number | null; category?: string | null;
}
export interface MarketPage { results: MarketItem[]; near?: MarketItem[]; page: number; total: number | null; hasMore: boolean }

/** `xiaoban://market/...`, already validated by the native side. */
export interface MarketLink { action: 'open' | 'install' | 'connect'; kind: MarketKind; namespace: string; slug: string; version: string | null; sha256: string | null }

export interface GroupMember extends SkillReview { name?: string; description?: string; state: 'installed' | 'pending'; kind: string }
export interface GroupReview { name: string; kind: string; sha256: string; members: GroupMember[] }

export interface UpdateInfo { kind: string; namespace: string; slug: string; installed: string; latest: string; hasUpdate: boolean; manifestSha256?: string }

export interface MarketDetail {
  item: MarketItem & { metadata?: { configs?: { name: string; config: Record<string, unknown> }[] } };
  versions: { version: string; yanked: boolean; manifestSha256: string }[];
  members?: { slug: string; name: string }[];
}

export const MARKET_SOURCE_KEY = 'xb.market.source';
export const MARKET_UPDATES_KEY = 'xb.market.autoUpdates';
export const SKILLHUB_KEY = 'xb.market.skillhub';

/** Short form of a digest for display; the full value is always available beside it. */
export function shortDigest(sha: string): string {
  return sha.length > 16 ? `${sha.slice(0, 8)}…${sha.slice(-6)}` : sha;
}

/** True when the market itself cannot be reached or is down, as opposed to answering "no such item". */
export function marketUnavailable(message: string): boolean {
  return message.includes('暂时无法连接小伴市场') || /小伴市场暂不可用（HTTP 5\d\d）/.test(message);
}

export type LinkState = 'ok' | 'newer' | 'mismatch' | 'missing';
export interface LinkVerdict { state: LinkState; /** version to review */ version: string | null; latest: string | null }

/**
 * Compares what the web link pinned with what the market says now. The link can only narrow things down:
 * a version it names must exist, not be withdrawn, and carry the digest it names. The real check
 * (recomputing the digest from the downloaded files) happens again when the version is reviewed.
 */
export function judgeLink(link: Pick<MarketLink, 'version' | 'sha256'>, detail: MarketDetail): LinkVerdict {
  const live = detail.versions.filter(v => !v.yanked);
  const latest = detail.item.version || live[0]?.version || null;
  if (!link.version) return live.length ? { state: 'ok', version: latest, latest } : { state: 'missing', version: null, latest: null };
  const pinned = live.find(v => v.version === link.version);
  if (!pinned) return { state: 'missing', version: null, latest };
  if (link.sha256 && pinned.manifestSha256.toLowerCase() !== link.sha256.toLowerCase()) return { state: 'mismatch', version: link.version, latest };
  if (latest && latest !== link.version) return { state: 'newer', version: latest, latest };
  return { state: 'ok', version: link.version, latest };
}

export interface FileChange { path: string; change: 'added' | 'changed' | 'removed'; size?: number; oldSize?: number; executableScript: boolean }

/** What an update changes, from the installed lock's file list and the new version's reviewed manifest. */
export function diffFiles(installed: { path: string; sha256: string }[], next: { path: string; size: number; sha256: string; executableScript: boolean }[]): FileChange[] {
  const before = new Map(installed.map(f => [f.path, f.sha256]));
  const changes: FileChange[] = [];
  for (const file of next) {
    const old = before.get(file.path);
    if (old === undefined) changes.push({ path: file.path, change: 'added', size: file.size, executableScript: file.executableScript });
    else if (old !== file.sha256) changes.push({ path: file.path, change: 'changed', size: file.size, executableScript: file.executableScript });
    before.delete(file.path);
  }
  for (const path of before.keys()) changes.push({ path, change: 'removed', executableScript: path.startsWith('scripts/') });
  const order = { added: 0, changed: 1, removed: 2 };
  return changes.sort((a, b) => order[a.change] - order[b.change] || a.path.localeCompare(b.path));
}

export const kindLabel = (kind: string | undefined) => kind === 'experts' || kind === 'expert' ? '专家' : kind === 'connectors' || kind === 'connector' ? '连接器' : '技能';
export const variantLabel = (v: Variant | undefined) => v === 'team' ? '专家团' : v === 'bundle' ? '技能套件' : '';
export const memberLabel = (v: Variant | undefined, n: number | null | undefined) => n ? (v === 'team' ? `${n} 位专家` : `${n} 个技能`) : '';

/** Persisted per-viewer preferences. Storage may be unavailable; defaults then apply. */
export function readPref(key: string, fallback: string): string {
  try { return localStorage.getItem(key) ?? fallback; } catch { return fallback; }
}
export function writePref(key: string, value: string) {
  try { localStorage.setItem(key, value); } catch { /* the choice just does not persist */ }
}
