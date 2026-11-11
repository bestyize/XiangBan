// Offline authoring: bake reversible puzzles with explicit solution witnesses.
import { readFileSync, writeFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
const root = new URL('../assets/game-packs/water-sort/', import.meta.url);
const ctx = {}; runInNewContext(readFileSync(new URL('engine.js', root), 'utf8'), ctx);
const R = ctx.XiabanWaterSort;
const names = ['第一口心动','草莓碰抹茶','午后三重奏','橘子汽水','莓好的一天','青提小憩','奶盖微风','香芋晚霞','街角招牌','请对号入杯','芒果双拼','蜜桃订单','抹茶专座','莓莓派对','三杯刚好','紫色星期五','果园来信','人气特调','一枚小封签','草莓来开锁','抹茶密语','柠檬钥匙','桃气满满','秘密花园','蓝莓谜题','云朵配方','解锁快乐','双重委托','日落订单','薄荷星球','夏夜微醺','彩虹来信','晴空之约','七分甜','满杯星光','金牌调饮师'];
const levels = [], fingerprints = new Set();
for (let i = 0; i < 36; i++) {
  const chapter = Math.floor(i / 9), stage = i % 9;
  const colors = chapter === 0 ? Math.min(4, 2 + Math.floor(stage / 3)) : Math.min(6, 3 + Math.floor(stage / 3) + (chapter === 3 ? 1 : 0));
  for (let attempt = 0; attempt < 400; attempt++) {
    const random = R.rng(20261112 + i * 7919 + attempt * 313);
    const board = Array.from({ length: colors + 2 }, (_, c) => c < colors ? Array(4).fill(c) : []);
    const reverse = [], seen = new Set([JSON.stringify(board)]);
    const hasSeal = chapter >= 2, sealIndex = colors + 1;
    for (let step = 0; step < 5 + stage * 2 + chapter * 3; step++) {
      const candidates = [];
      for (let from = 0; from < board.length; from++) for (let to = 0; to < board.length; to++) {
        if (from === to || !board[from].length || board[to].length === 4 || (hasSeal && (from === sealIndex || to === sealIndex))) continue;
        const color = board[from].at(-1);
        if (board[to].at(-1) === color) continue;
        let run = 1; while (run < board[from].length && board[from][board[from].length - 1 - run] === color) run++;
        for (let n = 1; n <= Math.min(run, 4 - board[to].length); n++) {
          if (n === run && n !== board[from].length) continue;
          const next = board.map(t => t.slice()); next[from].splice(-n); next[to].push(...Array(n).fill(color));
          if (!seen.has(JSON.stringify(next))) candidates.push({ from, to, next });
        }
      }
      if (!candidates.length) break;
      const c = candidates[Math.floor(random() * candidates.length)];
      board.splice(0, board.length, ...c.next); reverse.push([c.to, c.from]); seen.add(JSON.stringify(board));
    }
    const solution = reverse.reverse(), fingerprint = board.map(t => t.join('')).sort().join('|');
    if (solution.length < 4 || fingerprints.has(fingerprint) || board.filter(t => t.some(c => c !== t[0])).length < 2) continue;
    const orders = chapter === 1 || chapter === 3 ? Object.fromEntries(Array.from({ length: stage < 3 ? 1 : 2 }, (_, k) => [k, k])) : {};
    const seals = hasSeal ? { [sealIndex]: stage % colors } : {};
    const level = { id: 'c' + String(i + 1).padStart(2, '0'), chapter, name: names[i], colors, board, orders, seals, par: solution.length, solution };
    let state = R.initial(level);
    if (hasSeal && state.opened.length) continue;
    for (const [a, b] of solution) { state = R.pour(state, level, a, b); if (!state) break; }
    if (!state || !R.isWon(state, level)) continue;
    if (chapter === 0) { const shorter = R.solve(R.initial(level), level, 60000); if (!shorter || shorter.length < 4) continue; level.par = shorter.length; level.solution = shorter; }
    fingerprints.add(fingerprint); levels.push(level); break;
  }
  if (levels.length !== i + 1) throw Error('Unable to author level ' + (i + 1));
}
writeFileSync(new URL('levels.js', root), '// Generated offline by tools/generate-water-sort.mjs.\n' + 'globalThis.PourJoyLevels = Object.freeze(' + JSON.stringify(levels, null, 2) + ');\n');
console.log(`Baked ${levels.length} unique, witnessed puzzles; par ${levels.map(l => l.par).join('/')}`);
