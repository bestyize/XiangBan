import { describe, expect, it } from 'vitest';

// Every game pack must run when public/game-packs/<id>/ is copied out on its own.
const prefix = '../../public/game-packs/';
const files = Object.keys(import.meta.glob('../../public/game-packs/**/*', { query: '?url' })).map(f => f.slice(prefix.length));
const html = import.meta.glob('../../public/game-packs/*/index.html', { query: '?raw', import: 'default', eager: true }) as Record<string, string>;
const hosts = import.meta.glob('../../public/game-packs/*/desktop-host.js', { query: '?raw', import: 'default', eager: true }) as Record<string, string>;
const packs = Object.keys(html).map(f => f.slice(prefix.length).split('/')[0]).sort();

describe('self-contained desktop game packs', () => {
  it('finds all nine packs', () => {
    expect(packs).toEqual(['block-puzzle', 'dusk-glider', 'falling-blocks', 'ink-blade', 'jade-gomoku', 'merge-2048', 'pro-cube', 'snake-arena', 'water-sort']);
  });

  it.each(packs)('%s index.html only references files inside its own directory', id => {
    const text = html[`${prefix}${id}/index.html`];
    const refs = [...text.matchAll(/\b(?:src|href)\s*=\s*["']([^"']+)["']/gi)].map(m => m[1]);
    expect(refs.length).toBeGreaterThan(0);
    for (const ref of refs) {
      if (/^(#|data:|blob:|https?:|\/\/|javascript:|mailto:|tel:)/i.test(ref)) continue;
      expect(ref, `${id}: ${ref} must be relative to the pack`).not.toMatch(/^(\.\.\/|\/)/);
      const local = ref.split(/[?#]/, 1)[0].replace(/^\.\//, '');
      expect(files, `${id}: ${ref} must exist inside the pack`).toContain(`${id}/${local}`);
    }
  });

  it('ships the same desktop-host.js in every pack', () => {
    expect(Object.keys(hosts)).toHaveLength(packs.length);
    const [first, ...rest] = Object.values(hosts);
    expect(first.length).toBeGreaterThan(0);
    for (const copy of rest) expect(copy).toBe(first);
  });

  it('keeps no shared files at the game-packs root', () => {
    expect(files.filter(f => !f.includes('/'))).toEqual(['catalog.json']);
  });
});
