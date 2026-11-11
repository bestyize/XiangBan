(() => {
  'use strict';
  const CAPACITY = 4;
  const copy = s => ({ board: s.board.map(t => t.slice()), opened: s.opened.slice() });
  const full = t => t.length === CAPACITY && t.every(c => c === t[0]);
  function unlock(state, level) {
    for (const [tube, color] of Object.entries(level.seals || {})) {
      if (!state.opened.includes(+tube) && state.board.some(t => full(t) && t[0] === color)) state.opened.push(+tube);
    }
    return state;
  }
  const initial = level => unlock({ board: level.board.map(t => t.slice()), opened: [] }, level);
  const locked = (state, level, index) => Object.hasOwn(level.seals || {}, index) && !state.opened.includes(index);
  function getPour(state, level, from, to) {
    if (!Number.isInteger(from) || !Number.isInteger(to) || !state.board[from] || !state.board[to]) return { valid: false, reason: 'invalid' };
    if (from === to) return { valid: false, reason: 'same' };
    if (locked(state, level, from) || locked(state, level, to)) return { valid: false, reason: 'sealed' };
    const a = state.board[from], b = state.board[to];
    if (!a.length) return { valid: false, reason: 'empty' };
    if (b.length === CAPACITY) return { valid: false, reason: 'full' };
    const color = a[a.length - 1];
    if (b.length && b[b.length - 1] !== color) return { valid: false, reason: 'color' };
    let run = 1;
    while (run < a.length && a[a.length - 1 - run] === color) run++;
    return { valid: true, color, amount: Math.min(run, CAPACITY - b.length) };
  }
  function pour(state, level, from, to) {
    const move = getPour(state, level, from, to);
    if (!move.valid) return null;
    const next = copy(state);
    next.board[from].splice(-move.amount); next.board[to].push(...Array(move.amount).fill(move.color));
    return unlock(next, level);
  }
  const isWon = (s, level) => s.board.some(t => t.length) && s.board.every(t => !t.length || full(t)) && Object.entries(level.orders || {}).every(([i, c]) => full(s.board[i]) && s.board[i][0] === c);
  const completed = s => s.board.filter(full).length;
  function validate(state, level) {
    if (!state || !Array.isArray(state.board) || state.board.length !== level.board.length || !Array.isArray(state.opened) || state.opened.length > Object.keys(level.seals || {}).length) return false;
    if (state.opened.some(i => !Number.isInteger(i) || !Object.hasOwn(level.seals || {}, i)) || new Set(state.opened).size !== state.opened.length) return false;
    const counts = Array(level.colors).fill(0);
    for (let i = 0; i < state.board.length; i++) {
      const t = state.board[i];
      if (!Array.isArray(t) || t.length > CAPACITY || (locked(state, level, i) && t.length)) return false;
      for (const c of t) { if (!Number.isInteger(c) || c < 0 || c >= level.colors) return false; counts[c]++; }
    }
    return counts.every(n => n === CAPACITY);
  }
  const key = s => s.board.map(t => t.join('')).join('|') + '/' + s.opened.slice().sort().join(',');
  function createSearch(state, level, maxNodes = 40000) {
    const queue = [{ state: copy(state), parent: -1, move: null }], seen = new Set([key(state)]);
    let cursor = 0, result = null, done = false;
    const path = index => { const p = []; for (let i = index; queue[i].parent !== -1; i = queue[i].parent) p.push(queue[i].move); return p.reverse(); };
    return { step(batch = 250) {
      const until = cursor + batch;
      while (!done && cursor < queue.length && cursor < until) {
        const index = cursor++, s = queue[index].state;
        if (isWon(s, level)) { result = path(index); done = true; break; }
        for (let a = 0; a < s.board.length; a++) {
          if (!s.board[a].length) continue;
          if (!Object.keys(level.orders || {}).length && full(s.board[a])) continue;
          let emptyTried = false;
          for (let b = 0; b < s.board.length; b++) {
            if (!s.board[b].length && !Object.keys(level.orders || {}).length && !locked(s, level, b)) {
              if (emptyTried || s.board[a].every(c => c === s.board[a][0])) continue;
              emptyTried = true;
            }
            const next = pour(s, level, a, b); if (!next) continue;
            const k = key(next); if (seen.has(k)) continue;
            seen.add(k); queue.push({ state: next, parent: index, move: [a, b] });
            if (queue.length >= maxNodes) { done = true; break; }
          }
          if (done) break;
        }
      }
      if (cursor >= queue.length) done = true;
      return { done, path: result, visited: queue.length };
    } };
  }
  function solve(state, level, limit) { const search = createSearch(state, level, limit); let r; do { r = search.step(2000); } while (!r.done); return r.path; }
  function rng(seed) { let n = seed >>> 0; return () => { n += 0x6d2b79f5; let v = n; v = Math.imul(v ^ v >>> 15, v | 1); v ^= v + Math.imul(v ^ v >>> 7, v | 61); return ((v ^ v >>> 14) >>> 0) / 4294967296; }; }
  function shuffle(a, random) { const out = a.slice(); for (let i = out.length - 1; i > 0; i--) { const j = Math.floor(random() * (i + 1)); [out[i], out[j]] = [out[j], out[i]]; } return out; }
  function daily(code, levels) {
    if (!/^d\d{8}$/.test(code)) return null;
    const y = +code.slice(1, 5), m = +code.slice(5, 7), d = +code.slice(7, 9), date = new Date(Date.UTC(y, m - 1, d));
    if (y < 2020 || y > 2099 || date.getUTCFullYear() !== y || date.getUTCMonth() !== m - 1 || date.getUTCDate() !== d) return null;
    // The first release's pool is frozen so appending future chapters cannot change shared codes.
    const random = rng(+code.slice(1)), base = levels[9 + Math.floor(random() * 27)];
    const tubes = shuffle(base.board.map((_, i) => i), random), colors = shuffle(Array.from({ length: base.colors }, (_, i) => i), random);
    const positions = Object.fromEntries(tubes.map((old, i) => [old, i]));
    const map = obj => Object.fromEntries(Object.entries(obj || {}).map(([i, c]) => [positions[i], colors[c]]));
    return { ...base, id: code, name: `${m}月${d}日特调`, board: tubes.map(i => base.board[i].map(c => colors[c])), seals: map(base.seals), orders: map(base.orders), solution: base.solution.map(([a, b]) => [positions[a], positions[b]]) };
  }
  function stars(moves, par, hints) { return Math.min(hints ? 2 : 3, moves <= par ? 3 : moves <= par + 5 ? 2 : 1); }
  globalThis.XiabanWaterSort = Object.freeze({ CAPACITY, copy, full, initial, locked, getPour, pour, isWon, completed, validate, key, createSearch, solve, rng, daily, stars });
})();
