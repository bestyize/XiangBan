import { drawEffects } from '../../effects/draw.ts';
import { ParticleSystem } from '../../effects/sim.ts';
import type { EffectPack } from '../../effects/types.ts';

export const previewParams = { density: 2, speed: 1.4, size: 1 };

interface Still {
  content: string;
  pack: EffectPack;
  url?: string;
  queued: boolean;
  listeners: Set<(url: string) => void>;
}

// Session cache. Compare content within hash buckets as well, so even a hash
// collision or an imported pack replacing the same id cannot return a stale image.
const cache = new Map<number, Still[]>();
const queue: Still[] = [];
let scheduled = false;

function contentHash(content: string) {
  let hash = 0x811c9dc5;
  for (let i = 0; i < content.length; i++) hash = Math.imul(hash ^ content.charCodeAt(i), 0x01000193);
  return hash >>> 0;
}

function schedule() {
  if (scheduled || !queue.length) return;
  scheduled = true;
  // One render per task, including on the first visit. Never paint all cards in
  // the mounting/activation task or install per-card observers/animation loops.
  setTimeout(renderNext, 16);
}

function renderNext() {
  const still = queue.shift()!;
  if (!still.listeners.size) { finish(); return; }
  const canvas = document.createElement('canvas');
  canvas.width = 360; canvas.height = 202;
  const release = () => { canvas.width = 0; canvas.height = 0; finish(); };
  try {
    const context = canvas.getContext('2d', { alpha: true });
    if (!context) { release(); return; }
    const system = new ParticleSystem(still.pack);
    system.configure(canvas.width, canvas.height, previewParams);
    drawEffects(context, system);
    canvas.toBlob(blob => {
      try {
        if (blob) {
          still.url = URL.createObjectURL(blob);
          for (const ready of still.listeners) ready(still.url);
          still.listeners.clear();
        }
      } finally { release(); }
    }, 'image/webp', 0.85);
  } catch { release(); }

  function finish() {
    still.queued = false;
    scheduled = false;
    schedule();
  }
}

/** Subscribe to a shared still; disposal cancels delivery and skips unused queued renders. */
export function subscribeEffectStill(pack: EffectPack, ready: (url: string) => void): () => void {
  const content = JSON.stringify(pack);
  const hash = contentHash(content);
  const bucket = cache.get(hash) ?? [];
  let still = bucket.find(entry => entry.content === content);
  if (!still) {
    still = { content, pack: JSON.parse(content) as EffectPack, queued: false, listeners: new Set() };
    bucket.push(still); cache.set(hash, bucket);
  }
  if (still.url) { ready(still.url); return () => {}; }
  still.listeners.add(ready);
  if (!still.queued) { still.queued = true; queue.push(still); schedule(); }
  return () => { still.listeners.delete(ready); };
}
