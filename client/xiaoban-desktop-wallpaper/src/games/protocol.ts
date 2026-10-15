/** One game in the catalog. `base` is where its files live (`/game-packs/<id>/` for shipped games, the game-pack protocol for the user's own). */
export interface GameManifest { id: string; name: string; description: string; entry: string; cover?: string; runtimeProtocol: number; orientation?: string; save?: boolean; source?: 'builtin' | 'user'; base?: string }
export interface Metrics { score: number; lines?: number; maxTile?: number }
/** Compare JSON documents by value; serde_json can reorder object keys in native round trips. */
export function sameJson(left: unknown, right: unknown): boolean {
  if (left === right) return true;
  if (Array.isArray(left) || Array.isArray(right)) return Array.isArray(left) && Array.isArray(right) && left.length === right.length && left.every((value, i) => sameJson(value, right[i]));
  if (!left || !right || typeof left !== 'object' || typeof right !== 'object') return false;
  const a = left as Record<string, unknown>, b = right as Record<string, unknown>;
  const keys = Object.keys(a);
  return keys.length === Object.keys(b).length && keys.every(key => Object.hasOwn(b, key) && sameJson(a[key], b[key]));
}
export function metrics(data: unknown): Metrics | null {
  if (!data || typeof data !== 'object') return null;
  const d = data as Record<string, unknown>;
  const out: Record<string, number> = {};
  for (const key of ['score', 'lines', 'maxTile']) {
    if (d[key] === undefined && key !== 'score') continue;
    if (!Number.isSafeInteger(d[key]) || Number(d[key]) < 0) return null;
    out[key] = Number(d[key]);
  }
  return out as unknown as Metrics;
}
export function validSave(value: unknown): value is Record<string, unknown> {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return false;
  const prototype = Object.getPrototypeOf(value);
  if (prototype !== Object.prototype && prototype !== null) return false;
  try { return new TextEncoder().encode(JSON.stringify(value)).length <= 32768; } catch { return false; }
}
export class GameSession {
  ready = false;
  started = false;
  completed = false;
  record: Metrics | null = null;
  constructor(readonly id: string, readonly nonce: string, readonly source: Window) {}
  accept(event: Pick<MessageEvent, 'source' | 'data'>): Record<string, unknown> | null {
    const d = event.data;
    if (event.source !== this.source || !d || typeof d !== 'object' || d.channel !== 'xiaban.game-pack' || d.protocol !== 1 || d.nonce !== this.nonce || (d.gameId !== undefined && d.gameId !== this.id)) return null;
    if (d.type === 'game.ready') { this.ready = true; return d; }
    if (d.type === 'game.exit') return d;
    if (d.type === 'game.security') return d;
    if (!this.ready) return null;
    if (d.type === 'game.start') { this.started = true; this.completed = false; this.record = null; return d; }
    if (d.type === 'game.save') return validSave(d.data) ? d : null;
    if (['game.progress', 'game.result'].includes(d.type)) {
      const value = metrics(d);
      if (!this.started || this.completed || !value) return null;
      if (this.record && value.score < this.record.score) return null;
      this.record = value;
      if (d.type === 'game.result') this.completed = true;
      return d;
    }
    if (d.type === 'game.share') {
      const r = d.result;
      const value = metrics(r);
      if (!this.completed || !this.record || !value || r?.gameId !== this.id || ['score', 'lines', 'maxTile'].some(k => Number(value[k as keyof Metrics] || 0) !== Number(this.record?.[k as keyof Metrics] || 0))) return null;
      return d;
    }
    if (d.type === 'game.wallet') return d;
    return null;
  }
}
