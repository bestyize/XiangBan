import { SIZE, createGame, legalMoves, snapshot, swap } from './engine.mjs';
import { LEVELS } from './levels.mjs';
import { createMusic } from './music.mjs';
import { createRewards, rewardFor } from './rewards.mjs';

const xb = globalThis.xb;
const storageKey = 'pocket-sprites.v1';
const roundRulesVersion = 4;
const colors = ['红鸟', '星星', '绿芽', '水母', '紫狐', '云羊'];
const specialNames = { row: '横向火箭', col: '纵向火箭', bomb: '花团炸弹', rainbow: '虹彩鸟' };
const goalNames = { ice: '冰块', honey: '蜂蜜' };
const gameHelp = '滑动精灵，或依次点选相邻两格。';
const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)');
const byId = (id) => document.getElementById(id);
const stage = document.querySelector('.stage');
const screens = ['home', 'levels', 'game'].map(byId);
const board = byId('board');
const dialog = byId('dialog');
const hintButton = document.querySelector('[data-action="hint"]');
const pauseButton = document.querySelector('[data-action="pause"]');
const startButton = document.querySelector('[data-action="start"]');
const levelsButton = document.querySelector('#home [data-action="levels"]');
const levelsById = new Map(LEVELS.map((level) => [level.id, level]));

let record = { version: 1, stars: {}, muted: false, musicMuted: false, round: null };
let route = { screen: 'home' };
let ready = false;
let game;
let cells = [];
let focused = null;
let selected = null;
let hint = [];
let gesture;
let busy = false;
let turn = 0;
let hostHidden = false;
let dialogMode = '';
let leavePause = false;
let audioContext;
let music;
let rewardVoice;
let pendingSave = Promise.resolve();
const activeAnimations = new Set();
const voices = new Set();

function element(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text) node.textContent = text;
  return node;
}

function sprite(color) {
  const node = element('span', 'sprite');
  node.dataset.color = color;
  node.setAttribute('aria-hidden', 'true');
  return node;
}

function callHost(callback) {
  try {
    Promise.resolve(callback()).catch(() => {});
  } catch {
    // Host services may be unavailable in a standalone browser.
  }
}

function saveWarning(show) {
  byId('save-note').hidden = !show;
}

async function loadRecord() {
  try {
    const saved = xb?.storage
      ? await xb.storage.get(storageKey)
      : JSON.parse(localStorage.getItem(storageKey));
    if (saved?.version !== 1) return;

    record.muted = saved.muted === true;
    record.musicMuted = saved.musicMuted === true;
    record.round = saved.round;
    for (const level of LEVELS) {
      const stars = saved.stars?.[level.id];
      if (Number.isInteger(stars) && stars >= 1 && stars <= 3) record.stars[level.id] = stars;
    }
  } catch {
    saveWarning(true);
  }
}

function saveRecord() {
  const content = JSON.stringify(record);
  if (!xb?.storage) {
    try {
      localStorage.setItem(storageKey, content);
      saveWarning(false);
    } catch {
      saveWarning(true);
    }
    return;
  }

  pendingSave = pendingSave.then(async () => {
    try {
      await xb.storage.set(storageKey, JSON.parse(content));
      saveWarning(false);
    } catch {
      saveWarning(true);
    }
  });
}

function nextLevel(level) {
  return LEVELS[LEVELS.indexOf(level) + 1];
}

function firstUnfinished() {
  return LEVELS.find((level) => !record.stars[level.id]);
}

function isUnlocked(level) {
  const index = LEVELS.indexOf(level);
  return index === 0 || Boolean(record.stars[LEVELS[index - 1]?.id]);
}

function startLevel() {
  const saved = levelsById.get(record.round?.id);
  if (saved && isUnlocked(saved) && record.round.state?.status === 'playing') return saved;
  return firstUnfinished() ?? LEVELS[0];
}

function starsFor(gameState) {
  if (gameState.status !== 'won') return 0;
  return 1 + Number(gameState.moves >= 1) + Number(gameState.moves >= Math.max(3, Math.ceil(gameState.level.moves * .23)));
}

function restoreRound(level) {
  const saved = record.round?.state;
  // Completed stars stay valid when a level's board changes.
  if (record.round?.rulesVersion !== roundRulesVersion) return null;
  const size = level.size ?? SIZE;
  const cellCount = size * size;
  const hasBoard = Array.isArray(saved?.board) && saved.board.length === cellCount &&
    saved.board.every((tile, index) => tile === null || (
      level.mask[index] && level.colors.includes(tile?.color) &&
      (tile.special === null || Object.hasOwn(specialNames, tile.special))
    ));
  if (record.round?.id !== level.id || !hasBoard ||
      saved.ice?.length !== cellCount || saved.honey?.length !== cellCount ||
      saved.progress?.length !== level.goals.length || !Number.isInteger(saved.rngState) ||
      !Number.isInteger(saved.moves) || saved.moves < 0 || saved.moves > level.moves ||
      !Number.isInteger(saved.score) || saved.score < 0 ||
      !['playing', 'won', 'lost'].includes(saved.status)) return null;
  return Object.assign(createGame(level), saved);
}

function saveRound() {
  if (game.status === 'won') {
    const id = game.level.id;
    record.stars[id] = Math.max(record.stars[id] ?? 0, starsFor(game));
  }
  record.round = { id: game.level.id, rulesVersion: roundRulesVersion, state: snapshot(game) };
  saveRecord();
}

function renderHome() {
  startButton.disabled = !ready || LEVELS.length === 0;
  levelsButton.disabled = !ready;
  startButton.textContent = record.round?.state?.status === 'playing' ? '继续游戏' : '开始游戏';
}

function starsElement(count) {
  const stars = element('span', 'stars');
  stars.setAttribute('aria-hidden', 'true');
  for (let index = 0; index < 3; index++) {
    stars.append(element('span', index < count ? 'lit' : '', '★'));
  }
  return stars;
}

function renderLevels() {
  const cleared = LEVELS.filter((level) => record.stars[level.id]).length;
  byId('levels-progress').textContent = `已通关 ${cleared} / ${LEVELS.length}`;
  const current = firstUnfinished();
  const cards = LEVELS.map((level) => {
    const unlocked = isUnlocked(level);
    const stars = record.stars[level.id] ?? 0;
    const card = element('button', 'level');
    card.type = 'button';
    card.disabled = !unlocked;
    card.dataset.action = 'level';
    card.dataset.level = level.id;
    if (level === current && unlocked) card.dataset.current = 'true';
    const status = unlocked ? (stars ? `${stars} 星` : '可游玩') : '未解锁';
    card.setAttribute('aria-label', `第 ${level.id} 关，${level.name}，${status}`);
    card.append(element('span', 'level-number', String(level.id)));
    if (unlocked) {
      card.append(starsElement(stars));
    } else {
      const lock = element('span', 'icon icon-lock');
      lock.setAttribute('aria-hidden', 'true');
      card.append(lock);
    }
    return card;
  });
  byId('level-grid').replaceChildren(...cards);
}

function mountBoard(level) {
  const size = level.size ?? SIZE;
  cells = Array(size * size).fill(null);
  const indexes = level.mask.flatMap((active, index) => active ? [index] : []);
  const rowIndices = indexes.map((index) => Math.floor(index / size));
  const columnIndices = indexes.map((index) => index % size);
  const firstRow = Math.min(...rowIndices);
  const firstColumn = Math.min(...columnIndices);
  const rows = Math.max(...rowIndices) - firstRow + 1;
  const columns = Math.max(...columnIndices) - firstColumn + 1;
  board.parentElement.style.setProperty('--board-ratio', columns / rows);
  board.parentElement.style.setProperty('--display-columns', columns);
  board.style.setProperty('--rows', rows);
  board.style.setProperty('--columns', columns);
  board.setAttribute('aria-rowcount', rows);
  board.setAttribute('aria-colcount', columns);

  const rowNodes = Array.from({ length: rows }, (_, index) => {
    const row = element('div', 'board-row');
    row.setAttribute('role', 'row');
    row.setAttribute('aria-rowindex', index + 1);
    return row;
  });
  for (const index of indexes) {
    const row = Math.floor(index / size) - firstRow;
    const column = index % size - firstColumn;
    const cell = element('div', 'cell');
    cell.id = `cell-${index}`;
    cell.dataset.index = index;
    cell.dataset.tone = (row + column) % 2;
    cell.setAttribute('role', 'gridcell');
    cell.setAttribute('aria-colindex', column + 1);
    cell.style.gridArea = `${row + 1} / ${column + 1}`;
    const tile = element('span', 'tile');
    const tileImage = sprite(0);
    tile.setAttribute('aria-hidden', 'true');
    tile.append(tileImage);
    cell.append(tile);
    rowNodes[row].append(cell);
    cells[index] = { cell, tile, tileImage };
  }
  board.replaceChildren(...rowNodes);
  selected = null;
  focused = indexes.find((index) => game.board[index]) ?? indexes[0];
  byId('level-number').textContent = `第 ${level.id} 关`;
  byId('game-title').textContent = level.name;
}

function renderFocus() {
  for (const [index, part] of cells.entries()) {
    if (!part) continue;
    part.cell.setAttribute('aria-selected', index === selected);
    part.cell.dataset.focused = index === focused;
    part.cell.dataset.hint = hint.includes(index);
  }
  if (focused === null) board.removeAttribute('aria-activedescendant');
  else board.setAttribute('aria-activedescendant', `cell-${focused}`);
}

function renderCells(state) {
  for (const [index, part] of cells.entries()) {
    if (!part) continue;
    const { cell, tile, tileImage } = part;
    const occupant = state.board[index];
    const ice = state.ice[index];
    const honey = state.honey[index];
    tile.hidden = !occupant;
    if (occupant) tileImage.dataset.color = occupant.color;
    if (occupant?.special) tile.dataset.special = occupant.special;
    else delete tile.dataset.special;
    if (ice) cell.dataset.ice = ice;
    else delete cell.dataset.ice;
    if (honey) cell.dataset.honey = honey;
    else delete cell.dataset.honey;
    const label = honey ? `蜂蜜，${honey} 层` : [
      occupant ? colors[occupant.color] : '空格',
      occupant?.special && specialNames[occupant.special],
      ice && `${ice} 层冰`,
    ].filter(Boolean).join('，');
    cell.setAttribute('aria-label', label);
  }
}

function renderHud(state) {
  const moves = byId('moves');
  moves.textContent = state.moves;
  moves.dataset.low = state.moves <= 3;
  const list = byId('goals');
  const signature = `${game.level.id}:${state.progress.join(',')}`;
  if (list.dataset.progress === signature) return;
  list.dataset.progress = signature;
  const goals = game.level.goals.map((goal, index) => {
    const left = Math.max(0, goal.count - state.progress[index]);
    const item = element('li', 'goal');
    const name = goal.type === 'color' ? colors[goal.color] : goalNames[goal.type];
    item.dataset.progress = state.progress[index];
    item.dataset.complete = left === 0;
    item.setAttribute('aria-label', left ? `还需要 ${left} 个${name}` : `${name}目标已完成`);
    const icon = goal.type === 'color' ? sprite(goal.color) : element('span', `goal-icon ${goal.type}`);
    const count = element('span', 'goal-count', left ? String(left) : '✓');
    count.setAttribute('aria-hidden', 'true');
    item.append(icon, count);
    return item;
  });
  list.replaceChildren(...goals);
}

function feedback(message) {
  byId('feedback').textContent = message;
}

function renderGame() {
  renderCells(game);
  renderHud(game);
  const tutorial = game.level.tutorial;
  const fresh = game.moves === game.level.moves;
  hint = fresh ? tutorial?.swap ?? [] : [];
  feedback((fresh || !tutorial?.swap) && tutorial?.text ? tutorial.text : gameHelp);
  renderFocus();
}

function dialogButton(action, label, kind) {
  const button = element('button', kind, label);
  button.type = 'button';
  button.dataset.action = action;
  return button;
}

function updateSoundButton(button) {
  button.textContent = record.muted ? '开启声音' : '关闭声音';
}

function showDialog(mode) {
  const title = byId('dialog-title');
  const art = byId('dialog-art');
  const body = byId('dialog-text');
  const actions = byId('dialog-actions');
  if (mode === 'pause') {
    title.textContent = '游戏暂停';
    body.textContent = '';
    art.replaceChildren(sprite(2));
    const sound = dialogButton('sound', '', 'quiet');
    updateSoundButton(sound);
    actions.replaceChildren(
      dialogButton('resume', '继续游戏', 'primary'),
      dialogButton('retry', '重新开始', 'secondary'),
      dialogButton('map', '选择关卡', 'secondary'),
      sound,
      dialogButton('music', record.musicMuted ? '开启背景音乐' : '关闭背景音乐', 'quiet'),
    );
  } else if (game.status === 'won') {
    const next = nextLevel(game.level);
    title.textContent = next ? '关卡完成' : '花园通关';
    body.textContent = `${game.score.toLocaleString('zh-CN')} 分`;
    art.replaceChildren(starsElement(starsFor(game)));
    actions.replaceChildren(
      dialogButton(next ? 'next' : 'map', next ? '下一关' : '选择关卡', 'primary'),
      dialogButton(next ? 'map' : 'retry', next ? '选择关卡' : '再玩一次', 'secondary'),
    );
  } else {
    title.textContent = '步数用完了';
    body.textContent = '再试一次，看看还有哪些消除机会。';
    art.replaceChildren(sprite(5));
    actions.replaceChildren(
      dialogButton('retry', '再试一次', 'primary'),
      dialogButton('map', '选择关卡', 'secondary'),
    );
  }
  dialogMode = mode;
  if (!dialog.open) dialog.showModal();
}

function closeDialog() {
  dialogMode = '';
  if (dialog.open) dialog.close();
}

function setBusy(value) {
  busy = value;
  board.setAttribute('aria-busy', value);
  pauseButton.disabled = value;
  hintButton.disabled = value;
}

function resetGesture() {
  if (!gesture) return;
  const tile = cells[gesture.index]?.tile;
  tile?.style.removeProperty('--drag-x');
  tile?.style.removeProperty('--drag-y');
  delete cells[gesture.index]?.cell.dataset.motion;
  if (board.hasPointerCapture(gesture.id)) board.releasePointerCapture(gesture.id);
  gesture = null;
}

function cancelTurn() {
  rewardVoice?.stop();
  turn++;
  resetGesture();
  for (const animation of activeAnimations) animation.cancel();
  activeAnimations.clear();
  for (const part of cells) {
    if (!part) continue;
    delete part.cell.dataset.motion;
    delete part.cell.dataset.spark;
    delete part.cell.dataset.powerBirth;
  }
  for (const effect of byId('playfield').querySelectorAll('.effect-layer, .goal-flight')) effect.remove();
  setBusy(false);
}

function applyRoute(destination, options = {}) {
  cancelTurn();
  const level = levelsById.get(destination.id);
  if (destination.screen === 'game' && (!level || !isUnlocked(level))) {
    destination = { screen: 'home' };
    history.replaceState(destination, '');
  }
  route = destination;
  closeDialog();
  for (const screen of screens) screen.hidden = screen.id !== destination.screen;
  document.body.dataset.screen = destination.screen;
  stage.scrollTop = 0;

  if (destination.screen === 'game') {
    const changed = game?.level !== level;
    if (changed || options.fresh || !game) {
      const restored = options.fresh ? null : restoreRound(level);
      game = restored ?? createGame(level);
      mountBoard(level);
      if (!restored) saveRound();
      if (game.status === 'playing') {
        callHost(() => xb?.game?.start());
        if (game.score) callHost(() => xb?.game?.progress({ score: game.score }));
      }
    }
    renderGame();
    if (destination.paused && game.status === 'playing') showDialog('pause');
    else if (game.status !== 'playing') showDialog('result');
    else if (!options.initial) byId('game-title').focus({ preventScroll: true });
  } else if (destination.screen === 'levels') {
    renderLevels();
    if (!options.initial) byId('levels-title').focus({ preventScroll: true });
  } else {
    renderHome();
    if (!options.initial) byId('home-title').focus({ preventScroll: true });
  }
  syncAudio();
}

function navigate(destination, { replace = false, fresh = false } = {}) {
  history[replace ? 'replaceState' : 'pushState'](destination, '');
  applyRoute(destination, { fresh });
}

function openLevel(level, fresh = false) {
  if (!level || !isUnlocked(level)) return;
  const fromGame = route.screen === 'game';
  const parent = fromGame ? route.parent : route.screen;
  const saved = record.round?.id === level.id && record.round.state?.status === 'playing';
  navigate({ screen: 'game', id: level.id, parent }, { replace: fromGame, fresh: fresh || !saved });
}

function goToMap() {
  if (route.paused) {
    leavePause = true;
    history.back();
  } else if (route.parent === 'levels') {
    history.back();
  } else {
    navigate({ screen: 'levels' }, { replace: true });
  }
}

function onPopState(event) {
  if (!ready) return;
  const destination = event.state?.screen ? event.state : { screen: 'home' };
  if (leavePause) {
    leavePause = false;
    route = destination;
    goToMap();
    return;
  }
  applyRoute(destination);
}

function syncAudio() {
  if (!audioContext) return;
  music?.sync();
  if (hostHidden || document.hidden || record.muted || route.paused) {
    rewardVoice?.stop();
    for (const oscillator of voices) oscillator.stop();
    voices.clear();
    if (audioContext.state === 'running') audioContext.suspend().catch(() => {});
  } else if (audioContext.state !== 'running') {
    audioContext.resume().catch(() => {});
  }
}

function unlockAudio() {
  if (!audioContext && !record.muted) {
    const AudioContext = window.AudioContext || window.webkitAudioContext;
    if (!AudioContext) return;
    try {
      audioContext = new AudioContext();
      music = createMusic(audioContext, () => !record.muted && !record.musicMuted && !hostHidden && !document.hidden && !route.paused);
      rewardVoice = createRewards(audioContext,
        () => !record.muted && !hostHidden && !document.hidden && route.screen === 'game' && !route.paused,
        seconds => music?.duck(seconds));
    } catch {
      return;
    }
  }
  syncAudio();
  if (!record.muted) { music?.load(); rewardVoice?.load(); }
}

function tone(frequency, { end = frequency, type = 'sine', gain = .05, duration = .16, delay = 0 } = {}) {
  if (!audioContext || audioContext.state !== 'running' || record.muted || hostHidden || document.hidden) return;
  const start = audioContext.currentTime + delay;
  const oscillator = audioContext.createOscillator();
  const volume = audioContext.createGain();
  oscillator.type = type;
  oscillator.frequency.setValueAtTime(frequency, start);
  if (end !== frequency) oscillator.frequency.exponentialRampToValueAtTime(end, start + duration);
  volume.gain.setValueAtTime(.0001, start);
  volume.gain.exponentialRampToValueAtTime(gain, start + .008);
  volume.gain.exponentialRampToValueAtTime(gain * .37, start + duration * .38);
  volume.gain.exponentialRampToValueAtTime(.0001, start + duration);
  oscillator.connect(volume);
  volume.connect(audioContext.destination);
  voices.add(oscillator);
  oscillator.addEventListener('ended', () => {
    voices.delete(oscillator);
    oscillator.disconnect();
    volume.disconnect();
  }, { once: true });
  oscillator.start(start);
  oscillator.stop(start + duration);
}

function notes(frequencies, type = 'sine', gain = .06, duration = .16) {
  for (const [index, frequency] of frequencies.entries()) {
    tone(frequency, { type, gain, duration, delay: index * .05 });
  }
}

function clearSound(event) {
  const steps = [0, 2, 4, 7, 9, 12];
  const pitch = 585 * 2 ** (steps[Math.min(event.chain - 1, steps.length - 1)] / 12);
  const types = new Set(event.effects.map(({ special }) => special));
  if (types.has('rainbow')) {
    notes([pitch, pitch * 1.25, pitch * 1.5, pitch * 2], 'sine', .041, .20);
    tone(pitch / 3, { end: pitch / 2, gain: .023, duration: .20 });
  } else if (types.has('bomb')) {
    tone(172, { end: 51, type: 'triangle', gain: .07, duration: .19 });
    tone(420, { end: 128, gain: .038, duration: .13 });
    tone(pitch * 1.5, { gain: .034, delay: .04, duration: .19 });
  } else if (types.has('row') || types.has('col')) {
    tone(pitch / 2, { end: pitch * 1.6, type: 'triangle', gain: .044, duration: .13 });
    tone(pitch * 1.7, { gain: .033, delay: .05, duration: .16 });
  } else {
    tone(275, { end: 137, type: 'triangle', gain: .044, duration: .075 });
    tone(pitch, { gain: .052, delay: .016, duration: .17 });
    tone(pitch * 2.01, { gain: .018, delay: .02, duration: .11 });
    if (event.chain > 1 || event.created.length) {
      tone(pitch * 1.5, { gain: .038, delay: .057, duration: .16 });
    }
  }
}

function haptic(kind) {
  if (!hostHidden && xb?.can?.('haptics')) callHost(() => xb.haptics.impact(kind));
}

function canPlay() {
  return ready && route.screen === 'game' && !route.paused && !hostHidden &&
    !document.hidden && game?.status === 'playing' && !busy;
}

function neighbor(index, rowStep, columnStep) {
  const size = game.level.size ?? SIZE;
  const row = Math.floor(index / size) + rowStep;
  const column = index % size + columnStep;
  return row >= 0 && row < size && column >= 0 && column < size ? row * size + column : null;
}

function adjacent(first, second) {
  const size = game.level.size ?? SIZE;
  return Math.abs(first % size - second % size) +
    Math.abs(Math.floor(first / size) - Math.floor(second / size)) === 1;
}

function pointTo(index) {
  focused = index;
  if (selected !== null && selected !== index && adjacent(selected, index) && game.board[index]) {
    void playSwap(selected, index);
    return;
  }
  selected = selected === index || !game.board[index] ? null : index;
  renderFocus();
}

function pointerDown(event) {
  if (!canPlay() || gesture || event.button !== 0) return;
  const cell = event.target.closest('.cell');
  if (!cell) return;
  const index = Number(cell.dataset.index);
  gesture = { id: event.pointerId, index, x: event.clientX, y: event.clientY, width: cell.clientWidth };
  board.setPointerCapture(event.pointerId);
  board.focus({ preventScroll: true });
}

function pointerMove(event) {
  if (gesture?.id !== event.pointerId || !game.board[gesture.index] || !canPlay()) return;
  const dx = event.clientX - gesture.x;
  const dy = event.clientY - gesture.y;
  const limit = gesture.width * .38;
  const part = cells[gesture.index];
  const offset = (value) => `${Math.min(limit, Math.max(-limit, value))}px`;
  const horizontal = Math.abs(dx) > Math.abs(dy);
  part.cell.dataset.motion = 'true';
  part.tile.style.setProperty('--drag-x', horizontal ? offset(dx * .54) : '0px');
  part.tile.style.setProperty('--drag-y', horizontal ? '0px' : offset(dy * .54));
}

function pointerUp(event) {
  if (gesture?.id !== event.pointerId) return;
  const { index, x, y, width } = gesture;
  resetGesture();
  if (!canPlay()) return;
  const dx = event.clientX - x;
  const dy = event.clientY - y;
  if (Math.max(Math.abs(dx), Math.abs(dy)) < Math.max(11, width * .31)) {
    pointTo(index);
    return;
  }
  const horizontal = Math.abs(dx) >= Math.abs(dy);
  const other = neighbor(index, horizontal ? 0 : Math.sign(dy), horizontal ? Math.sign(dx) : 0);
  if (other !== null && game.board[index] && game.board[other]) {
    focused = other;
    void playSwap(index, other);
  } else {
    pointTo(index);
  }
}

function boardKey(event) {
  if (!canPlay()) return;
  if (event.key === 'Enter' || event.key === ' ') {
    event.preventDefault();
    if (focused !== null) pointTo(focused);
    return;
  }
  const directions = { ArrowUp: [-1, 0], ArrowDown: [1, 0], ArrowLeft: [0, -1], ArrowRight: [0, 1] };
  const direction = directions[event.key];
  if (!direction || focused === null) return;
  event.preventDefault();
  let next = neighbor(focused, ...direction);
  while (next !== null && !cells[next]) next = neighbor(next, ...direction);
  if (next !== null) {
    focused = next;
    renderFocus();
  }
}

function translated(from, to, entryDistance = 1, travel = 1) {
  const end = cells[to].cell.getBoundingClientRect();
  const start = from === null ? { left: end.left, top: end.top - end.height * entryDistance } :
    cells[from].cell.getBoundingClientRect();
  return `translate(${(start.left - end.left) * travel}px, ${(start.top - end.top) * travel}px)`;
}

const popColors = ['#ff947f', '#ffdf7f', '#b7e68d', '#8fdff4', '#d4adf2', '#f9dec3'];

function createEffect(layer, classes, point) {
  const effect = element('span', `effect ${classes}`);
  effect.style.setProperty('--effect-x', `${point.x}px`);
  effect.style.setProperty('--effect-y', `${point.y}px`);
  effect.style.setProperty('--effect-size', `${point.size}px`);
  layer.append(effect);
  return effect;
}

function createAnimation(node, keyframes, duration, easing, delay = 0) {
  if (reducedMotion.matches || hostHidden || document.hidden || !node.animate) return null;
  const animation = node.animate(keyframes, { duration, easing, delay, fill: 'both' });
  activeAnimations.add(animation);
  return animation;
}

function releaseAnimation(animation) {
  animation.cancel();
  activeAnimations.delete(animation);
}

function clearGeometry() {
  return cells.map((part) => {
    if (!part) return null;
    const { cell, tileImage } = part;
    return {
      x: cell.offsetLeft + cell.offsetWidth / 2,
      y: cell.offsetTop + cell.offsetHeight / 2,
      size: cell.offsetWidth,
      tint: popColors[Number(tileImage.dataset.color)],
    };
  });
}

function scatterChips(layer, point, index, chain, impact) {
  const burst = createEffect(layer, 'burst', point);
  burst.style.setProperty('--pop-color', point.tint);
  if (impact !== undefined) {
    burst.style.setProperty('--impact-delay', `${impact}ms`);
    burst.style.setProperty('--impact-duration', '125ms');
  }
  for (let chip = 0; chip < 3; chip++) {
    const shard = element('i');
    const angle = chip * Math.PI * 2 / 3 + index * .91;
    const distance = point.size * (.38 + chip * .12 + Math.min(chain - 1, 3) * .045);
    shard.style.setProperty('--chip-x', `${Math.cos(angle) * distance}px`);
    shard.style.setProperty('--chip-y', `${Math.sin(angle) * distance + 5}px`);
    shard.style.setProperty('--chip-size', `${Math.max(3, point.size * (.11 + chip * .02))}px`);
    shard.style.setProperty('--chip-spin', `${(chip - 1) * 130 + index * 17 % 100}deg`);
    burst.append(shard);
  }
}

function drawRainbowRays(layer, origin, cleared, points, animate) {
  const candidates = cleared.filter((index) => index !== origin);
  const stride = Math.max(1, Math.ceil(candidates.length / 7));
  for (let cursor = 0; cursor < candidates.length; cursor += stride) {
    const target = points[candidates[cursor]];
    const source = points[origin];
    const dx = target.x - source.x;
    const dy = target.y - source.y;
    const ray = createEffect(layer, 'ray', source);
    const angle = `rotate(${Math.atan2(dy, dx)}rad)`;
    ray.style.setProperty('--ray-length', `${Math.hypot(dx, dy)}px`);
    ray.style.setProperty('--ray-color', target.tint);
    animate(ray, [
      { transform: `${angle} scaleX(.03)`, opacity: 0 },
      { transform: angle, opacity: .82, offset: .31 },
      { transform: `${angle} scaleY(.25)`, opacity: 0 },
    ], 225, 'ease-out');
  }
}

function drawSpecials(layer, event, points, animate) {
  const size = game.level.size ?? SIZE;
  const affected = new Set([...event.cleared, ...event.iceHit, ...event.honeyHit]);
  const beamHits = new Map();
  function sweep(index, special) {
    const horizontal = special === 'row';
    const axis = horizontal ? 'x' : 'y';
    const translate = horizontal ? 'X' : 'Y';
    const extent = horizontal ? board.clientWidth : board.clientHeight;
    const point = points[index];
    const edgePadding = point.size * .32;
    const longest = Math.max(point[axis], extent - point[axis]) + edgePadding;
    const timePerPixel = 272 / longest;
    const launchDelay = 18;
    beamHits.set(index, { special, delay: launchDelay, launch: true });
    const launch = createEffect(layer, `beam-hit launch ${special}`, point);
    animate(launch, [
      { opacity: .12, transform: 'scale(.25)' },
      { opacity: .93, transform: 'scale(.94)', offset: .28 },
      { opacity: 0, transform: 'scale(1.46)' },
    ], 155);
    for (const direction of [-1, 1]) {
      const edge = direction < 0 ? -edgePadding : extent + edgePadding;
      const distance = edge - point[axis];
      const travel = Math.abs(distance) * timePerPixel;
      const duration = travel + 72;
      const angle = horizontal ? (direction > 0 ? 0 : 180) : direction * 90;
      const beam = createEffect(layer, `beam ${special}`, point);
      const transform = (position, scale) => `translate${translate}(${position}px) rotate(${angle}deg) scale(${scale})`;
      animate(beam, [
        { transform: transform(0, .72), opacity: .42 },
        { transform: transform(distance, 1), opacity: 1, offset: travel / duration },
        { transform: transform(distance + direction * point.size * .32, .8), opacity: 0 },
      ], duration, 'linear', launchDelay);
    }
    for (const target of affected) {
      const sameLine = horizontal ? Math.floor(target / size) === Math.floor(index / size) :
        target % size === index % size;
      if (!sameLine || target === index) continue;
      const delay = launchDelay + Math.abs(points[target][axis] - point[axis]) * timePerPixel;
      if (!beamHits.has(target) || beamHits.get(target).delay > delay) {
        beamHits.set(target, { special, delay });
      }
    }
  }

  for (const { index, tile } of event.created.slice(0, 3)) {
    const kind = tile.special === 'rainbow' ? 'rainbow' : 'birth';
    const ring = createEffect(layer, `ring ${kind}`, points[index]);
    animate(ring, [
      { transform: 'translate(-50%, -50%) scale(.27)', opacity: 0 },
      { transform: 'translate(-50%, -50%) scale(.58)', opacity: 1, offset: .48 },
      { transform: 'translate(-50%, -50%) scale(1.03)', opacity: 0 },
    ], 238, 'ease-out');
  }
  const drawn = new Set();
  let drewRays = false;
  const crossing = event.combo === 'line-line' ? event.effects[1].index : null;
  const effects = crossing === null ? event.effects : [
    { index: crossing, special: 'row' },
    { index: crossing, special: 'col' },
    ...event.effects.slice(2),
  ];
  for (const { index, special } of effects) {
    const key = special === 'row' ? `row${Math.floor(index / size)}` :
      special === 'col' ? `col${index % size}` : `${special}${index}`;
    if (drawn.has(key)) continue;
    drawn.add(key);
    const point = points[index];
    if (special === 'row' || special === 'col') {
      sweep(index, special);
    } else {
      const ring = createEffect(layer, `ring ${special}`, point);
      animate(ring, [
        { transform: 'translate(-50%, -50%) scale(.18) rotate(-18deg)', opacity: .12 },
        { transform: 'translate(-50%, -50%) scale(.57) rotate(14deg)', opacity: .9, offset: .34 },
        { transform: 'translate(-50%, -50%) scale(1.22) rotate(29deg)', opacity: 0 },
      ], 230, 'ease-out');
      if (special === 'rainbow' && !drewRays) {
        drawRainbowRays(layer, index, event.cleared, points, animate);
        drewRays = true;
      }
    }
    if (drawn.size === 6) break;
  }
  const flashes = [...beamHits].filter(([, hit]) => !hit.launch)
    .sort(([, first], [, second]) => first.delay - second.delay).slice(0, 24);
  for (const [index, { special, delay }] of flashes) {
    const flash = createEffect(layer, `beam-hit ${special}`, points[index]);
    animate(flash, [
      { opacity: 0, transform: 'scale(.42)' },
      { opacity: .94, transform: 'scale(1.06)', offset: .2 },
      { opacity: 0, transform: 'scale(1.24)' },
    ], 122, 'ease-out', delay);
  }
  return beamHits;
}

function flyGoals(event, points, artifacts, animate, beamHits) {
  const slots = byId('goals').children;
  const origin = board.getBoundingClientRect();
  const arrivals = [];
  for (const [index, goal] of game.level.goals.entries()) {
    const slot = slots[index];
    if (!slot || event.state.progress[index] <= Number(slot.dataset.progress)) continue;
    const isColor = goal.type === 'color';
    const sourceIndex = isColor
      ? event.cleared.find((cell) => Number(cells[cell].tileImage.dataset.color) === goal.color &&
        cells[cell].tile.dataset.special !== 'rainbow')
      : event[`${goal.type}Hit`].find((cell) => event.state[goal.type][cell] === 0);
    if (sourceIndex === undefined) continue;
    const impact = beamHits.get(sourceIndex)?.delay ?? 0;
    const source = points[sourceIndex];
    const destination = slot.firstElementChild.getBoundingClientRect();
    const flight = element('span', 'goal-flight');
    const size = Math.max(23, source.size * .75);
    const startX = origin.left + source.x;
    const startY = origin.top + source.y;
    const dx = destination.left + destination.width / 2 - startX;
    const dy = destination.top + destination.height / 2 - startY;
    flight.setAttribute('aria-hidden', 'true');
    flight.style.left = `${startX - size / 2}px`;
    flight.style.top = `${startY - size / 2}px`;
    flight.style.setProperty('--flight-size', `${size}px`);
    flight.append(isColor ? sprite(goal.color) : element('span', `goal-icon ${goal.type}`));
    byId('playfield').append(flight);
    artifacts.push(flight);
    arrivals.push(index);
    animate(flight, [
      { opacity: 0, transform: 'scale(.6)' },
      { opacity: 1, transform: 'scale(1.1)', offset: .18 },
      { opacity: 1, transform: `translate(${dx * .47 + 18}px, ${dy * .43 - 21}px) scale(.88)`, offset: .57 },
      { opacity: .3, transform: `translate(${dx}px, ${dy}px) scale(.42)` },
    ], Math.min(245, 430 - impact), 'cubic-bezier(.43,.08,.36,.98)', impact);
    if (arrivals.length === 3) break;
  }
  return arrivals;
}

function popWave(event, animate, artifacts, sparks) {
  const points = clearGeometry();
  const layer = element('span', 'effect-layer');
  layer.setAttribute('aria-hidden', 'true');
  artifacts.push(layer);
  const beamHits = drawSpecials(layer, event, points, animate);
  for (const index of event.cleared) {
    const part = cells[index];
    if (part.tile.hidden) continue;
    const impact = beamHits.get(index)?.delay;
    animate(part.tile, [
      { opacity: 1, transform: 'scale(1)' },
      { opacity: 1, transform: 'translateY(2px) scale(.79,.70)', offset: .27 },
      { opacity: 1, transform: 'translateY(-4px) scale(1.2,1.12)', offset: .48 },
      { opacity: 0, transform: 'translateY(-7px) scale(.27) rotate(12deg)' },
    ], impact === undefined ? 178 - Math.min(event.chain - 1, 4) * 5 : 132,
    'ease-out', impact ?? index % 4 * 7);
  }
  const hit = [...new Set([...event.iceHit, ...event.honeyHit, ...event.cleared])];
  const stride = Math.max(1, Math.ceil(hit.length / 6));
  for (let cursor = 0; cursor < hit.length; cursor += stride) {
    const index = hit[cursor];
    const cell = cells[index].cell;
    const point = points[index];
    if (event.iceHit.includes(index)) point.tint = '#a9eaf8';
    if (event.honeyHit.includes(index)) point.tint = '#ffc162';
    const impact = beamHits.get(index)?.delay;
    if (impact !== undefined) {
      cell.style.setProperty('--impact-delay', `${impact}ms`);
      cell.style.setProperty('--impact-duration', '125ms');
    } else {
      cell.style.removeProperty('--impact-delay');
      cell.style.removeProperty('--impact-duration');
    }
    cell.style.setProperty('--pop-color', point.tint);
    cell.dataset.spark = 'true';
    sparks.push(cell);
    scatterChips(layer, point, index, event.chain, impact);
  }
  board.append(layer);
  return flyGoals(event, points, artifacts, animate, beamHits);
}

function bumpGoals(arrivals) {
  for (const index of arrivals) {
    const target = byId('goals').children[index];
    if (!target) continue;
    const animation = createAnimation(target, [
      { transform: 'scale(.94)' },
      { transform: 'scale(1.11)', backgroundColor: '#8e784f', offset: .28 },
      { transform: 'scale(1)' },
    ], 180, 'ease-out');
    if (animation) void animation.finished.then(() => releaseAnimation(animation), () => {});
  }
}

async function celebratePower(created, token) {
  const names = { row: '横向高能！', col: '纵向高能！', bomb: '爆炸高能！', rainbow: '虹彩鸟诞生！' };
  music?.duck(1.1);
  notes([659, 880, 1319], 'sine', .105, .25);
  feedback(created.map(({ tile }) => names[tile.special]).join(' · '));
  if (reducedMotion.matches || hostHidden || document.hidden) return;
  const layer = element('span', 'effect-layer power-layer');
  layer.setAttribute('aria-hidden', 'true');
  board.append(layer);
  const points = clearGeometry();
  const animations = [];
  for (const { index, tile } of created.slice(0, 3)) {
    const point = points[index];
    const halo = createEffect(layer, 'ring power-birth', point);
    const label = element('span', 'power-label', names[tile.special]);
    label.style.left = `${Math.max(60, Math.min(board.clientWidth - 60, point.x))}px`;
    label.style.top = `${Math.max(8, point.y - point.size)}px`;
    layer.append(label);
    cells[index].cell.dataset.powerBirth = 'true';
    for (const [node, frames] of [
      [cells[index].tile, [{transform:'scale(.55)',filter:'brightness(2)'},{transform:'scale(1.38)',filter:'brightness(1.5)',offset:.38},{transform:'scale(1.14)',offset:.78},{transform:'scale(1)',filter:'brightness(1)'}]],
      [halo, [{transform:'translate(-50%,-50%) scale(.25)',opacity:0},{transform:'translate(-50%,-50%) scale(.8)',opacity:1,offset:.35},{transform:'translate(-50%,-50%) scale(1.3)',opacity:0}]],
      [label, [{transform:'translate(-50%,8px) scale(.7)',opacity:0},{transform:'translate(-50%,0) scale(1.08)',opacity:1,offset:.25},{transform:'translate(-50%,-4px)',opacity:1,offset:.8},{transform:'translate(-50%,-8px)',opacity:0}]],
    ]) {
      const animation = createAnimation(node, frames, 440, 'cubic-bezier(.2,.7,.25,1)');
      if (animation) animations.push(animation);
    }
  }
  await Promise.allSettled(animations.map(animation => animation.finished));
  animations.forEach(releaseAnimation);
  layer.remove();
  if (token === turn) for (const {index} of created) delete cells[index]?.cell.dataset.powerBirth;
}

async function animatePhase(event, token) {
  const animations = [];
  const moving = [];
  const sparks = [];
  const artifacts = [];
  let arrivals = [];
  const animate = (node, frames, duration, easing = 'ease-out', delay = 0) => {
    const animation = createAnimation(node, frames, duration, easing, delay);
    if (animation) animations.push(animation);
  };
  if (event.type === 'swap' || event.type === 'revert') {
    const isRevert = event.type === 'revert';
    for (const [from, to] of [[event.a, event.b], [event.b, event.a]]) {
      const part = cells[from];
      part.cell.dataset.motion = 'true';
      moving.push(part.cell);
      animate(part.tile, [
        { transform: 'translate(0, 0)' },
        { transform: `${translated(to, from, 1, .58)} scale(1.08,.94)`, offset: .52 },
        { transform: `${translated(to, from, 1, 1.07)} scale(1.04,.97)`, offset: .83 },
        { transform: translated(to, from) },
      ], isRevert ? 150 : 140, 'cubic-bezier(.32,.02,.32,1)');
    }
    if (isRevert) tone(214, { end: 137, type: 'triangle', gain: .033, duration: .11 });
    else tone(386, { end: 313, type: 'triangle', gain: .024, duration: .07 });
  } else if (event.type === 'clear') {
    music?.duck(event.activated.length || event.created.length ? 1 : .65);
    if (!reducedMotion.matches && !hostHidden && !document.hidden) {
      arrivals = popWave(event, animate, artifacts, sparks);
    }
    clearSound(event);
    const messages = [];
    if (event.chain >= 2) messages.push(`${event.chain} 连消`);
    if (event.created.length) messages.push(`生成${specialNames[event.created[0].tile.special]}`);
    if (messages.length) {
      feedback(messages.join(' · '));
      animate(byId('feedback'), [
        { opacity: .7, transform: 'translateY(3px) scale(.96)' },
        { opacity: 1, color: '#ffdf91', transform: 'translateY(-2px) scale(1.04)', offset: .38 },
        { transform: 'translateY(0)' },
      ], 195, 'ease-out');
    }
    if (event.activated.length) haptic('light');
  } else if (event.type === 'fall') {
    const size = game.level.size ?? SIZE;
    renderCells(event.state);
    const fresh = new Set(event.drops.filter(({ from }) => from === null).map(({ to }) => to));
    for (const { from, to } of event.drops) {
      const part = cells[to];
      let entryDistance = .9;
      if (from === null) {
        for (let above = to - size; fresh.has(above); above -= size) entryDistance++;
      }
      const distance = from === null ? entryDistance : (to - from) / size;
      animate(part.tile, [
        { transform: translated(from, to, entryDistance), opacity: from === null ? .24 : 1 },
        { transform: 'translateY(0) scale(1.065,.91)', opacity: 1, offset: .73 },
        { transform: 'translateY(-2px) scale(.986,1.017)', opacity: 1, offset: .89 },
        { transform: 'translateY(0)', opacity: 1 },
      ], 176 + Math.min(distance, 3) * 9, 'cubic-bezier(.27,.48,.56,1)', to % 4 * 10);
    }
  } else if (event.type === 'shuffle') {
    renderCells(event.state);
    feedback('棋盘已重新排列。');
    notes([370, 450, 540], 'sine', .038);
    animate(board, [{ opacity: .55, transform: 'scale(.985)' }, { opacity: 1, transform: 'scale(1)' }], 220);
  }
  await Promise.allSettled(animations.map((animation) => animation.finished));
  for (const animation of animations) releaseAnimation(animation);
  for (const artifact of artifacts) artifact.remove();
  if (token !== turn) return;
  for (const cell of moving) delete cell.dataset.motion;
  for (const cell of sparks) delete cell.dataset.spark;
  renderCells(event.state);
  renderHud(event.state);
  bumpGoals(arrivals);
  if (event.type === 'clear' && event.created.length && token === turn) await celebratePower(event.created, token);
}

async function playSwap(a, b) {
  if (!canPlay() || !adjacent(a, b) || !game.board[a] || !game.board[b]) return;
  const result = swap(game, a, b);
  selected = null;
  hint = [];
  renderFocus();
  if (!result.events.length) return;
  setBusy(true);
  const token = ++turn;
  if (result.valid) {
    saveRound();
    callHost(() => xb?.game?.progress({ score: game.score }));
    if (game.status !== 'playing') callHost(() => xb?.game?.result({ score: game.score }));
    feedback(gameHelp);
  }
  for (const event of result.events) {
    if (token !== turn) return;
    await animatePhase(event, token);
  }
  if (token !== turn) return;
  renderCells(game);
  renderHud(game);
  setBusy(false);
  if (result.valid) rewardVoice?.play(rewardFor(result.events, game.status), game.status === 'won');
  if (!result.valid) {
    const tutorial = game.level.tutorial;
    if (tutorial?.swap && game.moves === game.level.moves) {
      hint = tutorial.swap;
      renderFocus();
      feedback(tutorial.text);
    } else {
      feedback('交换后要连成至少三个相同的精灵。');
    }
  }
  if (game.status !== 'playing') {
    if (game.status === 'won') {
      notes([587, 740, 880, 1175], 'sine', .07, .28);
      if (xb?.can?.('haptics')) callHost(() => xb.haptics.notify('success'));
    } else {
      notes([370, 294], 'sine', .045, .22);
    }
    showDialog('result');
  }
}

function showHint() {
  if (!canPlay()) return;
  const tutorial = game.level.tutorial;
  const pair = game.moves === game.level.moves && tutorial?.swap ? tutorial.swap : legalMoves(game)[0];
  if (!pair) return;
  selected = null;
  hint = pair;
  renderFocus();
  feedback('试试交换发光的两格。');
  notes([670], 'sine', .045, .16);
}

function onAction(event) {
  const button = event.target.closest('button[data-action]');
  if (!button || button.disabled || !ready) return;
  const action = button.dataset.action;
  if (action === 'start') openLevel(startLevel());
  else if (action === 'levels') navigate({ screen: 'levels' });
  else if (action === 'back' || action === 'resume') history.back();
  else if (action === 'level') openLevel(levelsById.get(Number(button.dataset.level)));
  else if (action === 'pause' && canPlay()) navigate({ ...route, paused: true });
  else if (action === 'hint') showHint();
  else if (action === 'map') goToMap();
  else if (action === 'next') openLevel(nextLevel(game.level));
  else if (action === 'retry') {
    if (route.paused) {
      game = createGame(game.level);
      mountBoard(game.level);
      saveRound();
      callHost(() => xb?.game?.start());
      history.back();
    } else openLevel(game.level, true);
  } else if (action === 'sound') {
    record.muted = !record.muted;
    unlockAudio();
    saveRecord();
    updateSoundButton(button);
  } else if (action === 'music') {
    record.musicMuted = !record.musicMuted;
    button.textContent = record.musicMuted ? '开启背景音乐' : '关闭背景音乐';
    unlockAudio();
    saveRecord();
  }
}

function lifecycle() {
  resetGesture();
  syncAudio();
  if (hostHidden || document.hidden) {
    for (const animation of activeAnimations) animation.finish();
  }
}

function connectEvents() {
  stage.addEventListener('click', onAction);
  document.addEventListener('click', unlockAudio);
  document.addEventListener('keydown', unlockAudio);
  window.addEventListener('popstate', onPopState);
  document.addEventListener('visibilitychange', lifecycle);
  board.addEventListener('pointerdown', pointerDown);
  board.addEventListener('pointermove', pointerMove);
  board.addEventListener('pointerup', pointerUp);
  const cancelPointer = (event) => { if (gesture?.id === event.pointerId) resetGesture(); };
  board.addEventListener('pointercancel', cancelPointer);
  board.addEventListener('lostpointercapture', cancelPointer);
  board.addEventListener('keydown', boardKey);
  dialog.addEventListener('close', () => {
    if (dialog.open) return;
    if (dialogMode === 'pause') history.back();
    if (dialogMode === 'result') goToMap();
  });
  xb?.on?.('hide', () => { hostHidden = true; lifecycle(); });
  xb?.on?.('show', () => { hostHidden = false; lifecycle(); });
}

async function init() {
  connectEvents();
  renderHome();
  document.body.dataset.screen = 'home';
  requestAnimationFrame(() => {
    requestAnimationFrame(() => callHost(() => xb?.ready()));
  });
  await loadRecord();
  ready = true;
  const initial = history.state?.screen ? history.state : { screen: 'home' };
  history.replaceState(initial, '');
  applyRoute(initial, { initial: true });
}

void init();
