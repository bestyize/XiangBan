(() => {
  'use strict';
  const R = globalThis.XiabanWaterSort, levels = globalThis.PourJoyLevels, training = globalThis.PourJoyTraining;
  const integer = (v, min, max) => Number.isInteger(v) && v >= min && v <= max;
  const getLevel = id => levels.find(l => l.id === id) || training.find(l => l.id === id) || R.daily(id, levels);
  const fresh = () => ({ version: 2, records: {}, daily: {}, sound: true, music: true, trainingDone: false, session: null });
  const unlocked = data => { let i = 0; while (i < levels.length - 1 && data.records[levels[i].id]) i++; return i; };
  function replay(level, moves, extraAt = null) {
    if (extraAt !== null && (!integer(extraAt, 0, moves.length) || level.id.startsWith('t'))) return null;
    let state = R.initial(level);
    const history = [];
    for (let i = 0; i <= moves.length; i++) {
      if (i === extraAt) {
        if (R.isWon(state, level)) return null;
        state = { ...state, board: [...state.board, []] };
      }
      if (i === moves.length) break;
      const move = moves[i];
      if (!Array.isArray(move) || move.length !== 2 || R.isWon(state, level)) return null;
      const next = R.pour(state, level, ...move); if (!next) return null;
      history.push(state); state = next;
    }
    return { state, history };
  }
  function cleanRecord(v) {
    return v && integer(v.moves, 1, 4096) && integer(v.stars, 1, 3) && typeof v.assisted === 'boolean'
      ? { moves: v.moves, stars: v.stars, assisted: v.assisted } : null;
  }
  function sanitize(raw) {
    const out = fresh(); if (!raw || raw.version !== 2) return out;
    out.sound = raw.sound === true;
    out.music = raw.music === true;
    for (const level of levels) { const record = cleanRecord(raw.records?.[level.id]); if (record) out.records[level.id] = record; }
    // A record beyond a missing predecessor must not silently unlock the campaign.
    let gap = false;
    for (const level of levels) { if (!out.records[level.id]) gap = true; if (gap) delete out.records[level.id]; }
    if (raw.daily && typeof raw.daily === 'object') {
      for (const id of Object.keys(raw.daily).filter(id => /^d\d{8}$/.test(id)).sort().slice(-30)) {
        const record = cleanRecord(raw.daily[id]); if (record && getLevel(id)) out.daily[id] = record;
      }
    }
    const s = raw.session, level = typeof s?.id === 'string' ? getLevel(s.id) : null;
    const extraAt = s?.extraAt === undefined ? null : s.extraAt;
    if (level && ['campaign', 'daily', 'challenge', 'training'].includes(s.mode) && Array.isArray(s.moves) && s.moves.length <= 4096 && integer(s.hints, 0, 4096) &&
        (s.mode !== 'campaign' || (level.id[0] === 'c' && levels.indexOf(level) <= unlocked(out))) &&
        (s.mode !== 'daily' || level.id[0] === 'd') && (s.mode === 'training') === level.id.startsWith('t') && replay(level, s.moves, extraAt)) {
      out.session = { id: s.id, mode: s.mode, moves: s.moves.map(m => m.slice()), hints: s.hints, extraAt };
    }
    out.trainingDone = raw.trainingDone === true || Object.keys(out.records).length > 0 || out.session?.mode === 'campaign';
    return out;
  }
  function finish(data, session) {
    const level = getLevel(session.id), replayed = level && replay(level, session.moves, session.extraAt);
    if (!replayed || !R.isWon(replayed.state, level)) return null;
    const assisted = session.hints > 0 || session.extraAt != null;
    const result = { moves: session.moves.length, stars: R.stars(session.moves.length, level.par, assisted), assisted };
    if (session.mode === 'training' && level === training.at(-1)) data.trainingDone = true;
    const table = session.mode === 'campaign' ? data.records : session.mode === 'daily' ? data.daily : null;
    if (table) {
      const prev = table[level.id];
      // Keep the best star result; ties prefer fewer moves and then no assistance.
      if (!prev || result.stars > prev.stars || (result.stars === prev.stars && (result.moves < prev.moves || (result.moves === prev.moves && !result.assisted)))) table[level.id] = result;
      if (session.mode === 'daily') for (const id of Object.keys(table).sort().slice(0, -30)) delete table[id];
    }
    return { ...result, id: level.id, mode: session.mode };
  }
  function today(now = new Date()) { return 'd' + now.getFullYear() + String(now.getMonth() + 1).padStart(2, '0') + String(now.getDate()).padStart(2, '0'); }
  globalThis.PourJoyProgress = Object.freeze({ fresh, sanitize, unlocked, replay, finish, today, getLevel });
})();
