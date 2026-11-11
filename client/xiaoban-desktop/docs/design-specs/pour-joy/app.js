(() => {
  'use strict';
  const $ = (s, r = document) => r.querySelector(s);
  const $$ = (s, r = document) => [...r.querySelectorAll(s)];
  const colors = [
    { name: '草莓', symbol: '莓', color: '#ff527a' },
    { name: '抹茶', symbol: '茶', color: '#73ca38' },
    { name: '芒果', symbol: '芒', color: '#ffbe22' },
    { name: '香芋', symbol: '芋', color: '#a66bf0' },
  ];
  // Selected fixed puzzles from the existing game; bottom-to-top order is preserved.
  const levels = [
    { id: 'first-01', title: '第一口心动', subtitle: 'CHAPTER 01 · 街角初遇', board: [[0, 1, 0, 1], [0, 1, 0, 1], [], []], target: 8, flavors: 2 },
    { id: 'afternoon-02', title: '午后三重奏', subtitle: 'CHAPTER 01 · 街角初遇', board: [[], [0, 1, 0, 2], [2, 1, 0, 1], [1, 2], [0, 2]], target: 12, flavors: 3 },
    { id: 'berry-03', title: '莓好的一天', subtitle: 'CHAPTER 01 · 街角初遇', board: [[2, 3, 0], [1, 1, 3, 2], [2, 0, 1, 3], [0], [1, 2, 3, 0], []], target: 16, flavors: 4 },
  ];
  const icons = {
    sound: '<path d="M11 5 6 9H3v6h3l5 4zM15 8c3 2 3 6 0 8m3-11c5 4 5 10 0 14"/>',
    mute: '<path d="M11 5 6 9H3v6h3l5 4zM16 9l5 6m0-6-5 6"/>',
    undo: '<path d="M8 5 3 10l5 5M3 10h11a6 6 0 0 1 0 12" transform="translate(1 -2)"/>',
    hint: '<path d="M9 18h6m-5 3h4M8 14a6 6 0 1 1 8 0c-1 1-1 2-1 2H9s0-1-1-2Z"/>',
    reset: '<path d="M3 10a9 9 0 1 1 2 8M3 4v6h6"/>',
    cup: '<path d="M5 7h14l-2 14H7zM4 4h16M13 4l3-3"/>',
    book: '<path d="M12 5v16M12 5C9 2 4 3 2 4v15c4-2 7-1 10 2 3-3 6-4 10-2V4c-2-1-7-2-10 1Z"/>',
    arrow: '<path d="M4 12h16m-6-6 6 6-6 6"/>',
    download: '<path d="M12 3v12m-5-5 5 5 5-5M4 16v5h16v-5"/>',
    home: '<path d="m3 10 9-7 9 7v11H3zM9 21v-8h6v8"/>',
  };
  const icon = (name) => `<svg class="icon" viewBox="0 0 24 24" aria-hidden="true">${icons[name] || icons.cup}</svg>`;
  const drink = (color = '#ff6891', label = 'joy') => `<div class="drink" style="--drink-color:${color}" aria-hidden="true"><div class="straw"></div><div class="drink-body"><div class="pearls"></div></div><div class="lid"></div><div class="cup-label">${label}<small>POUR JOY</small></div></div>`;
  const clone = b => b.map(t => t.slice());
  const screen = $('#screen');
  const challenge = new URLSearchParams(location.search).get('challenge');
  let levelIndex = Math.max(0, levels.findIndex(l => l.id === challenge));
  if (!challenge) levelIndex = 2;
  let board = clone(levels[levelIndex].board), moves = 0, undoStack = [], selected = -1;
  let view = 'play', busy = false, sound = false, hintPair = null, won = false, toastTimer, audio;
  let lastWin = null;
  const earned = new Set();
  function toast(text) {
    clearTimeout(toastTimer); $('#toast').textContent = text; $('#toast').classList.add('show');
    toastTimer = setTimeout(() => $('#toast').classList.remove('show'), 2800);
  }
  function openDialog(title, content) {
    $('#infoDialog h2').textContent = title; $('#dialogContent').innerHTML = content;
    if (!$('#infoDialog').open) $('#infoDialog').showModal();
  }
  $('.dialog-close').onclick = () => $('#infoDialog').close();
  $('#infoDialog').addEventListener('click', e => { if (e.target === $('#infoDialog')) $('#infoDialog').close(); });
  function go(next, push = true) {
    if (busy) return;
    view = ['home', 'play', 'journal', 'share'].includes(next) ? next : 'play';
    if (push && location.hash !== '#' + view) history.pushState({ view }, '', '#' + view);
    $$('.review-tabs button').forEach(b => { b.classList.toggle('active', b.dataset.view === view); b.setAttribute('aria-pressed', String(b.dataset.view === view)); });
    render(); screen.scrollTop = 0;
  }
  window.addEventListener('popstate', () => go(location.hash.slice(1) || 'play', false));
  document.addEventListener('click', e => { const b = e.target.closest('[data-view]'); if (b) go(b.dataset.view); });
  $('.brand').onclick = e => { e.preventDefault(); go('play'); };
  $('#hostMenu').onclick = () => openDialog('宿主胶囊', '<p>这里为小伴宿主胶囊的视觉占位。正式接入使用宿主提供的菜单与退出能力；游戏内容避让顶部区域。</p><button class="primary" id="returnHome">返回小店</button>');
  document.addEventListener('click', e => { if (e.target.id === 'returnHome') { $('#infoDialog').close(); go('home'); } });
  function full(t) { return t.length === 4 && t.every(v => v === t[0]); }
  function isWon(b) { return b.every(t => t.length === 0 || full(t)); }
  function moveInfo(b, from, to) {
    if (from === to || !b[from]?.length || !b[to] || b[to].length >= 4) return null;
    const color = b[from].at(-1);
    if (b[to].length && b[to].at(-1) !== color) return null;
    let run = 1; while (run < b[from].length && b[from][b[from].length - 1 - run] === color) run++;
    return { amount: Math.min(run, 4 - b[to].length), color };
  }
  function moved(b, from, to) { const info = moveInfo(b, from, to); if (!info) return null; const next = clone(b); next[from].splice(-info.amount); next[to].push(...Array(info.amount).fill(info.color)); return next; }
  function render() { ({ play: renderPlay, home: renderHome, journal: renderJournal, share: renderShare })[view](); }
  function renderPlay() {
    const focusedTube = document.activeElement?.dataset?.tube;
    const level = levels[levelIndex];
    screen.innerHTML = `<div class="game-page"><div class="game-topline"><button class="chapter-tag" id="levelPicker"><i></i> 街角初遇 <span>⌄</span></button><button class="icon-button" id="soundButton" aria-label="${sound ? '关闭' : '开启'}音效" title="${sound ? '关闭' : '开启'}音效">${icon(sound ? 'sound' : 'mute')}</button></div><div class="level-heading"><div><h2>${level.title}</h2><p>第 0${levelIndex + 1} 关</p></div><div class="step-count"><strong id="moveCount">${String(moves).padStart(2, '0')}</strong><span>步</span></div></div><div class="order-strip"><span class="completion-count">${board.filter(full).length}<small> / ${level.flavors}</small></span><div class="order-icons">${colors.slice(0, level.flavors).map((c, i) => `<span class="order-dot ${board.some(t => full(t) && t[0] === i) ? 'done' : ''}" style="--flavor:${c.color}" title="${c.name}"><i></i>${c.symbol}</span>`).join('')}</div></div>${won ? winPanel() : '<div class="board-area"><div class="board" aria-label="调饮杯盘"></div></div><p class="game-tip" role="status" aria-live="polite">' + (selected < 0 ? (moves === 0 ? '同色相接 · 空杯中转' : '') : `选择接收杯`) + '</p>'}<div class="tools"><button class="tool" id="undo" ${!undoStack.length || won ? 'disabled' : ''}><span>${icon('undo')}</span>撤销</button><button class="tool" id="hint" ${won ? 'disabled' : ''}><span>${icon('hint')}</span>灵感</button><button class="tool" id="reset"><span>${icon('reset')}</span>重调</button></div><div class="game-bottom"><button data-view="home">← 回小店</button><button id="rules">玩法 ⓘ</button></div></div>`;
    if (!won) renderBoard();
    if (focusedTube !== undefined) $(`[data-tube="${focusedTube}"]`)?.focus({ preventScroll: true });
    $('#soundButton').onclick = () => { sound = !sound; if (sound) chime(true); renderPlay(); toast(sound ? '音效已开启' : '音效已关闭'); };
    $('#undo').onclick = undo;
    $('#hint').onclick = showHint;
    $('#reset').onclick = () => { if (!moves) { start(levelIndex); return; } openDialog('重新调这一杯？', '<p>当前关卡的步数与杯子会回到初始状态。</p><button class="primary" id="confirmReset">重新开始</button><button class="secondary" id="cancelReset">继续这一杯</button>'); $('#confirmReset').onclick = () => { $('#infoDialog').close(); start(levelIndex); }; $('#cancelReset').onclick = () => $('#infoDialog').close(); };
    $('#rules').onclick = rules;
    $('#levelPicker').onclick = levelPicker;
    if (won) { $('#nextLevel').onclick = () => start((levelIndex + 1) % levels.length); }
  }
  function renderBoard() {
    const grid = $('.board'); if (!grid) return;
    grid.innerHTML = board.map((tube, i) => `<button class="tube-button ${selected === i ? 'selected' : ''} ${selected >= 0 && moveInfo(board, selected, i) ? 'target' : ''} ${hintPair?.includes(i) ? 'hint' : ''} ${full(tube) ? 'complete' : ''}" data-tube="${i}" aria-pressed="${selected === i}" aria-label="${i + 1} 号杯，${tube.length ? '从底到顶：' + tube.map(v => colors[v].name).join('、') : '空杯'}"><span class="tube-shadow"></span><span class="glass"><span class="liquid-stack">${tube.map(v => `<span class="liquid" style="--flavor:${colors[v].color}">${colors[v].symbol}</span>`).join('')}</span>${full(tube) ? '<span class="complete-seal">✓</span>' : ''}</span><span class="cup-caption">${full(tube) ? colors[tube[0]].name + ' · 完成' : '0' + (i + 1)}</span></button>`).join('');
    grid.querySelectorAll('button').forEach(b => b.onclick = () => selectTube(Number(b.dataset.tube)));
  }
  async function selectTube(i) {
    if (busy || won) return;
    hintPair = null;
    if (selected === i) { selected = -1; renderPlay(); return; }
    if (selected < 0) { if (!board[i].length) { toast('先选有饮料的杯子'); return; } selected = i; renderPlay(); return; }
    const from = selected, info = moveInfo(board, from, i);
    if (!info) {
      const b = $(`[data-tube="${i}"]`); b?.classList.add('shake');
      toast(board[i].length === 4 ? '这杯满啦' : '顶层不同色');
      return;
    }
    busy = true;
    // Mutating controls stay disabled until the animation and state update finish.
    $$('.screen button').forEach(b => b.disabled = true);
    undoStack.push({ board: clone(board), moves });
    await animatePour(from, i, info.color);
    board = moved(board, from, i); moves++; selected = -1;
    won = isWon(board);
    if (won) { earned.add(levelIndex); lastWin = { levelIndex, moves, assisted: hintsUsed > 0, target: levels[levelIndex].target }; }
    busy = false; renderPlay();
    if (won) { chime(true); toast('新风味已解锁！'); }
    else if (full(board[i])) { chime(true); toast(colors[info.color].name + '完成 ✓'); }
  }
  async function animatePour(from, to, color) {
    if (matchMedia('(prefers-reduced-motion: reduce)').matches) { chime(false); return; }
    const source = $(`[data-tube="${from}"] .glass`), target = $(`[data-tube="${to}"] .glass`);
    const a = source.getBoundingClientRect(), b = target.getBoundingClientRect();
    const ghost = source.cloneNode(true); ghost.classList.add('pour-ghost');
    Object.assign(ghost.style, { left: a.left + 'px', top: a.top + 'px', width: a.width + 'px', height: a.height + 'px' });
    document.body.append(ghost); source.style.opacity = '.15';
    const dx = b.left - a.left - 42, dy = b.top - a.top - 54;
    const motion = ghost.animate([{ transform: 'translate(0,0) rotate(0deg)' }, { transform: `translate(${dx}px,${dy}px) rotate(57deg)`, offset: .4 }, { transform: `translate(${dx}px,${dy}px) rotate(57deg)`, offset: .75 }, { transform: 'translate(0,0) rotate(0deg)' }], { duration: 680, easing: 'ease-in-out', fill: 'forwards' });
    const stream = document.createElement('div'); stream.className = 'pour-stream';
    Object.assign(stream.style, { left: b.left + b.width / 2 + 'px', top: b.top - 22 + 'px', height: '38px', background: colors[color].color });
    const timer = setTimeout(() => { document.body.append(stream); chime(false); stream.animate([{ opacity: 0, transform: 'scaleY(0)' }, { opacity: .8, transform: 'scaleY(1)' }, { opacity: 0, transform: 'scaleY(.9)' }], { duration: 280 }); }, 260);
    try { await motion.finished; } catch (_) { /* DOM removal may cancel the visual only. */ }
    clearTimeout(timer); ghost.remove(); stream.remove(); source.style.opacity = '';
  }
  function chime(success) {
    if (!sound) return;
    try {
      audio ||= new (window.AudioContext || window.webkitAudioContext)(); audio.resume();
      const now = audio.currentTime;
      if (success) [523, 659, 784].forEach((freq, i) => { const o = audio.createOscillator(), g = audio.createGain(); o.type = 'sine'; o.frequency.value = freq; g.gain.setValueAtTime(.0001, now + i * .07); g.gain.exponentialRampToValueAtTime(.035, now + i * .07 + .01); g.gain.exponentialRampToValueAtTime(.0001, now + i * .07 + .25); o.connect(g).connect(audio.destination); o.start(now + i * .07); o.stop(now + i * .07 + .3); });
      else { const buffer = audio.createBuffer(1, audio.sampleRate * .25, audio.sampleRate); const data = buffer.getChannelData(0); for (let i = 0; i < data.length; i++) data[i] = (Math.random() * 2 - 1) * .12; const src = audio.createBufferSource(), f = audio.createBiquadFilter(), g = audio.createGain(); src.buffer = buffer; f.type = 'lowpass'; f.frequency.value = 1400; g.gain.setValueAtTime(.3, now); g.gain.exponentialRampToValueAtTime(.001, now + .25); src.connect(f).connect(g).connect(audio.destination); src.start(); }
    } catch (_) { /* Sound is optional; gameplay remains available. */ }
  }
  function undo() { if (busy || !undoStack.length || won) return; const prev = undoStack.pop(); board = prev.board; moves = prev.moves; selected = -1; hintPair = null; renderPlay(); }
  let hintsUsed = 0;
  function start(index) { if (busy) return; levelIndex = index; board = clone(levels[index].board); selected = -1; moves = 0; undoStack = []; hintPair = null; won = false; hintsUsed = 0; go('play'); }
  function solve(initial, maxNodes = 65000) {
    const key = b => b.map(t => t.join('')).sort().join('|');
    const queue = [{ b: clone(initial), path: [] }], seen = new Set([key(initial)]);
    for (let qi = 0; qi < queue.length && qi < maxNodes; qi++) {
      const { b, path } = queue[qi]; if (isWon(b)) return path;
      for (let a = 0; a < b.length; a++) {
        if (!b[a].length || full(b[a])) continue;
        let triedEmpty = false;
        for (let z = 0; z < b.length; z++) {
          if (!b[z].length) { if (triedEmpty) continue; triedEmpty = true; if (b[a].every(v => v === b[a][0])) continue; }
          const next = moved(b, a, z); if (!next) continue;
          const k = key(next); if (seen.has(k)) continue;
          seen.add(k); queue.push({ b: next, path: [...path, [a, z]] });
          if (queue.length >= maxNodes) return null;
        }
      }
    }
    return null;
  }
  function showHint() {
    if (busy || won) return;
    const path = solve(board);
    if (!path?.length) { toast('这一步可能卡住了，试试撤销最近几步。'); return; }
    hintsUsed++; hintPair = path[0]; selected = path[0][0]; renderPlay();
    $('.game-tip').textContent = `灵感来了：把 ${hintPair[0] + 1} 号杯倒进 ${hintPair[1] + 1} 号杯。`;
  }
  function rules() { openDialog('只要记住这三件事', '<ul><li>先点来源杯，再点目标杯；只有空杯或顶层同色的杯子能接。</li><li>一次倒出顶端连续的同口味饮料，直到倒完或目标杯装满。每杯容量 4 层。</li><li>所有非空杯都装满，且各自只有一种风味，即可完成。</li></ul><p>没有倒计时。撤销不限次数，「灵感」给出下一步。每层保留汉字标识，不依赖颜色识别。</p>'); }
  function levelPicker() { openDialog('街角初遇', '<p>设计样稿开放 3 个可解关卡。正式版按章节解锁，关卡内容与难度曲线另行制作。</p><div class="level-list">' + levels.map((l, i) => `<button data-level="${i}">0${i + 1} · ${l.title}<span>${l.flavors} 种风味 ${earned.has(i) ? '✓' : '→'}</span></button>`).join('') + '</div>'); $$('[data-level]').forEach(b => b.onclick = () => { $('#infoDialog').close(); start(Number(b.dataset.level)); }); }
  function winPanel() { return `<div class="win-panel"><div class="win-badge">✳</div><h3>完美出杯！</h3><p>${moves} 步 · 解锁新风味</p><div style="height:130px;display:grid;place-items:center">${drink()}</div><button class="primary" data-view="share">收下小票 ${icon('arrow')}</button><button class="secondary" id="nextLevel">再调一杯 →</button></div>`; }
  function shopSVG() { return `<svg viewBox="0 0 390 280" role="img" aria-label="街角饮品店，珊瑚红遮阳棚与薄荷绿橱窗"><defs><linearGradient id="wall" x2="0" y2="1"><stop stop-color="#f8e8cd"/><stop offset="1" stop-color="#ebd6b9"/></linearGradient><linearGradient id="window" x2="0" y2="1"><stop stop-color="#63c9c7"/><stop offset="1" stop-color="#baf1dc"/></linearGradient></defs><ellipse cx="196" cy="254" rx="148" ry="17" fill="#e1d1b4" opacity=".45"/><path d="M54 245h286" stroke="#c5b291"/><path d="M92 87h209v157H92z" fill="url(#wall)" stroke="#cfb392"/><path d="M102 83h188v-22q0-15-15-15H117q-15 0-15 15z" fill="#f1dcc0" stroke="#d4b794"/><rect x="129" y="55" width="134" height="29" rx="5" fill="#fff9e9"/><text x="196" y="75" text-anchor="middle" fill="#e95135" font-size="16" font-weight="700" font-family="Microsoft YaHei,sans-serif">倒杯快乐</text><path d="m87 85-14 27h244l-14-27z" fill="#f9edd8" stroke="#bd907d"/><path d="m87 85-14 27h25l11-27m25 0-7 27h25l3-27m25 0v27h25l-3-27m25 0 7 27h25l-11-27m25 0 14 27h-25l-11-27" fill="#f45b47"/><path d="M73 112h25v9q-12 13-25 0zm50 0h25v9q-12 13-25 0zm50 0h25v9q-12 13-25 0zm50 0h25v9q-12 13-25 0zm50 0h25v9q-12 13-25 0z" fill="#f45b47"/><path d="M98 112h25v9q-12 13-25 0zm50 0h25v9q-12 13-25 0zm50 0h25v9q-12 13-25 0zm50 0h25v9q-12 13-25 0zm50 0h19v9q-8 13-19 0z" fill="#fff2dc"/><rect x="109" y="141" width="104" height="72" rx="3" fill="url(#window)" stroke="#b69c7c" stroke-width="4"/><path d="M161 141v72m-52-34h104" stroke="#f7edd6" stroke-width="3"/><path d="m117 153 18-8m-18 16 30-13m24 43 30-14" stroke="#ffffff50" stroke-width="5"/><rect x="225" y="141" width="58" height="103" rx="2" fill="#baa58b"/><rect x="230" y="146" width="48" height="90" fill="#ede0c7"/><rect x="235" y="151" width="38" height="50" rx="18" fill="#a6b19a"/><circle cx="270" cy="215" r="3" fill="#8b6250"/><rect x="102" y="211" width="120" height="7" rx="2" fill="#b3926e"/><path d="M125 210v-18h13v18m15 0v-25h14v25m17 0v-17h13v17" fill="#e6b1a4" stroke="#f6dfc5" stroke-width="2"/><path d="m130 192 3-10m27 3 3-10m26 18 3-10" stroke="#f45b47" stroke-width="2"/><path d="m230 244-4 7h60l-3-7" fill="#c6ae8e"/><path d="m64 201-8 45m31-45 9 45" stroke="#a17e5f" stroke-width="3"/><rect x="58" y="195" width="36" height="45" rx="4" fill="#627968" stroke="#a98866" stroke-width="3"/><text x="76" y="212" text-anchor="middle" fill="#fff2d5" font-size="7">TODAY</text><path d="M68 222h16m-13 5h10" stroke="#eee4c8" stroke-width="2"/><path d="M313 224h26l-4 21h-18z" fill="#b9816a"/><path d="M324 225v-30m0 21q-24-1-13-20 14 2 13 20m2-5q-2-25 16-16 3 15-16 16m-1-12q-15-12-3-24 12 8 3 24" fill="#6dbd43"/><circle cx="310" cy="53" r="18" fill="#ffbd2f" opacity=".8"/><path d="M49 82h14m-7-7v14m275 76h10m-5-5v10" stroke="#c8a06e" stroke-width="1.5"/><text x="195" y="270" text-anchor="middle" fill="#a78f72" font-size="7" letter-spacing="3"></text></svg>`; }
  function nav(active) { return `<nav class="bottom-nav" aria-label="游戏导航"><button data-view="home" class="${active === 'home' ? 'active' : ''}">${icon('home')}小店</button><button data-view="journal" class="${active === 'journal' ? 'active' : ''}">${icon('book')}风味手帐</button></nav>`; }
  function renderHome() {
    screen.innerHTML = `<div class="home-page"><div class="screen-header"><div class="wordmark">倒杯快乐<small>POUR JOY CLUB</small></div><span class="pill">✳ 营业中</span></div><div class="home-copy"><h2>来一杯<span>快乐！</span></h2></div><div class="shop-art">${shopSVG()}</div><div class="home-action"><button class="primary" id="continue">${won ? '再调一杯' : '继续调一杯'} ${icon('arrow')}</button><div class="continue-info"><span>街角初遇 · 第 0${levelIndex + 1} 杯</span><span>${levels[levelIndex].title}</span></div></div><button class="daily-card" id="daily"><span class="daily-mark">✳</span><span><strong>好友同题</strong><small>挑战 003</small></span><span class="arrow">↗</span></button>${nav('home')}</div>`;
    $('#continue').onclick = () => won ? start((levelIndex + 1) % levels.length) : go('play');
    $('#daily').onclick = () => { start(2); toast('同题挑战 003：分享链接，邀朋友调同一杯。'); };
  }
  function renderJournal(season = 'spring') {
    const entries = season === 'spring' ? [
      { name: '第一口心动', line: earned.has(0) ? '本次试玩已收集' : '完成第 01 杯收集', color: '#ffc330', open: earned.has(0) },
      { name: '午后三重奏', line: earned.has(1) ? '本次试玩已收集' : '完成第 02 杯收集', color: '#79cb43', open: earned.has(1) },
      { name: '莓好的一天', line: earned.has(2) ? '本次试玩已收集' : '完成第 03 杯收集', color: '#ff6891', open: earned.has(2) },
      { name: '云朵香芋', line: '下一章 · 概念预览', color: '#a66bf0', open: false },
    ] : [{ name: '海盐落日', line: '夏日章节 · 概念预览', color: '#e4af77', open: false }, { name: '青提气泡', line: '夏日章节 · 概念预览', color: '#acbb86', open: false }, { name: '椰椰假日', line: '夏日章节 · 概念预览', color: '#dcc7a6', open: false }, { name: '蓝色星期天', line: '夏日章节 · 概念预览', color: '#90adb6', open: false }];
    screen.innerHTML = `<div class="journal-page"><div class="journal-header"><h2>风味手帐</h2></div><div class="collection-progress"><div class="track"><span style="width:${earned.size / 4 * 100}%"></span></div><span>${String(earned.size).padStart(2, '0')} / 04 本季风味</span></div><div class="journal-tabs"><button data-season="spring" class="${season === 'spring' ? 'active' : ''}">街角初遇</button><button data-season="summer" class="${season === 'summer' ? 'active' : ''}">海边来信</button></div><div class="sticker-grid">${entries.map((e, i) => `<button class="sticker ${e.open ? '' : 'locked'}" data-sticker="${i}"><span class="tiny">NO. 0${i + 1} / POUR JOY</span><span class="sticker-art">${drink(e.color)}</span><h3>${e.name}</h3><p>${e.line}</p></button>`).join('')}</div><p class="collection-message">${earned.size ? '新风味已收集' : '通关收集风味'} · 本次试玩</p>${nav('journal')}</div>`;
    $$('[data-season]').forEach(b => b.onclick = () => renderJournal(b.dataset.season));
    $$('[data-sticker]').forEach(b => b.onclick = () => { const i = Number(b.dataset.sticker), e = entries[i]; if (e.open) { openDialog(e.name, `<div style="height:150px;display:grid;place-items:center">${drink(e.color)}</div><p style="text-align:center">本次试玩亲手调好的快乐，已收入你的风味手帐。</p>`); } else if (season === 'spring' && i < 3) { openDialog('去收集「' + e.name + '」', '<p>完成对应关卡，就能点亮这枚饮品贴纸。开始会重置当前未完成关卡。</p><button class="primary" id="collectNow">去调这一杯</button>'); $('#collectNow').onclick = () => { $('#infoDialog').close(); start(i); }; } else toast('这是后续章节的风味概念，样稿暂未开放。'); });
  }
  function receiptData() { return lastWin || { levelIndex: 2, moves: 16, assisted: false, target: 16, sample: true }; }
  function renderShare() {
    const d = receiptData(), level = levels[d.levelIndex];
    screen.innerHTML = `<div class="share-page"><div class="share-heading"><h2>快乐小票</h2><span>${d.sample ? '示例小票' : '本次通关'}</span></div><div class="receipt"><span class="eyebrow">POUR JOY · 快乐营业中</span><h3>今日快乐小票</h3><div class="receipt-art">${drink()}<span class="stamp">快乐<br>已打包</span></div><h4 class="receipt-name">${level.title}</h4><div class="receipt-data"><div><strong>${String(d.moves).padStart(2, '0')}</strong><span>调饮步数</span></div><div><strong>${level.flavors}</strong><span>风味归位</span></div><div><strong>100%</strong><span>今日好心情</span></div></div><div class="receipt-line"><span>同题编号</span><span>PJ-00${d.levelIndex + 1} · ${d.assisted ? '灵感助调' : '自由调饮'}</span></div><div class="barcode" aria-hidden="true"></div><div class="receipt-footer">${d.sample ? 'DESIGN SAMPLE / ' : ''}KEEP THIS LITTLE JOY.</div></div><div class="share-actions"><button class="primary" id="saveReceipt">${icon('download')}保存快乐小票</button><button class="secondary" id="copyChallenge">挑战好友 ↗</button></div><p class="share-disclaimer">${d.sample ? '设计示例' : '本次通关记录'} · 同题试玩</p></div>`;
    $('#saveReceipt').onclick = saveReceipt;
    $('#copyChallenge').onclick = async () => {
      const url = new URL(location.href); url.search = ''; url.searchParams.set('challenge', level.id); url.hash = 'play';
      if (url.protocol === 'file:') { toast('本地文件地址不能发给朋友，请使用在线预览。'); return; }
      const text = `来「倒杯快乐」调一杯${level.title}。同一道题，看看你的解法：${url.href}`;
      try { await navigator.clipboard.writeText(text); toast('同题邀请已复制，可以发给朋友了。'); }
      catch (_) { openDialog('复制同题邀请', '<p>浏览器暂不允许直接复制，可手动复制下面的链接。</p><input class="dialog-input" id="challengeLink" readonly>'); $('#challengeLink').value = url.href; $('#challengeLink').select(); }
    };
  }
  function saveReceipt() {
    const d = receiptData(), level = levels[d.levelIndex];
    const canvas = document.createElement('canvas'); canvas.width = 1080; canvas.height = 1440; const ctx = canvas.getContext('2d');
    const rect = (x, y, w, h, r, fill) => { ctx.fillStyle = fill; ctx.beginPath(); ctx.roundRect(x, y, w, h, r); ctx.fill(); };
    const text = (t, x, y, size, color = '#e95135', weight = '400') => { ctx.fillStyle = color; ctx.font = `${weight} ${size}px "Microsoft YaHei","Segoe UI",sans-serif`; ctx.textAlign = 'center'; ctx.fillText(t, x, y); };
    ctx.fillStyle = '#ffbe71'; ctx.fillRect(0, 0, 1080, 1440);
    rect(90, 80, 900, 1280, 8, '#fffcf3');
    for (let x = 90; x < 990; x += 80) rect(x, 80, 40, 14, 0, '#e95135');
    for (let x = 90; x < 990; x += 32) { ctx.fillStyle = '#ffbe71'; ctx.beginPath(); ctx.moveTo(x, 1360); ctx.lineTo(x + 16, 1345); ctx.lineTo(x + 32, 1360); ctx.fill(); }
    text('POUR JOY CLUB', 540, 171, 24, '#e95135', '600'); text('今日快乐小票', 540, 252, 60, '#432f31', '700'); 
    ctx.save(); ctx.translate(540, 513); ctx.rotate(-.12); rect(31, -160, 15, 115, 4, '#ef5441'); rect(-92, -85, 184, 245, 38, '#ff7297'); rect(-92, -85, 184, 89, [25, 25, 0, 0], '#f3e4c9'); rect(-102, -97, 204, 25, 12, '#fff3db'); rect(-50, 4, 100, 100, 50, '#fffaed'); text('joy', 0, 66, 39, '#e95135', '700'); ctx.restore();
    ctx.save(); ctx.translate(786, 440); ctx.rotate(.2); ctx.strokeStyle = '#ef5441'; ctx.lineWidth = 4; ctx.beginPath(); ctx.arc(0, 0, 70, 0, Math.PI * 2); ctx.stroke(); text('快乐', 0, -6, 24); text('已打包', 0, 29, 24); ctx.restore();
    text(level.title, 540, 741, 39, '#432f31', '700'); text('这杯快乐，我调的。', 540, 793, 24, '#d9784a');
    ctx.strokeStyle = '#cbb99e'; ctx.lineWidth = 2; ctx.setLineDash([8, 8]); ctx.beginPath(); ctx.moveTo(160, 840); ctx.lineTo(920, 840); ctx.moveTo(160, 999); ctx.lineTo(920, 999); ctx.stroke(); ctx.setLineDash([]);
    [String(d.moves).padStart(2, '0'), String(level.flavors), '100%'].forEach((t, i) => text(t, 280 + i * 260, 914, 58)); ['调饮步数', '风味归位', '今日好心情'].forEach((t, i) => text(t, 280 + i * 260, 958, 21, '#9b8068'));
     text(`同题 PJ-00${d.levelIndex + 1} · ${d.assisted ? '灵感助调' : '自由调饮'}`, 540, 1110, 23, '#896c58');
    for (let x = 365; x < 715; x += 13) rect(x, 1164, x % 3 ? 4 : 8, 40, 0, '#6c4a3b'); text(d.sample ? 'DESIGN SAMPLE · 设计示例' : 'KEEP THIS LITTLE JOY.', 540, 1240, 19, '#a58b75'); text('倒杯快乐 · 一杯刚刚好的颜色', 540, 1296, 22, '#e95135');
    canvas.toBlob(blob => { if (!blob) { toast('图片生成失败，请重试。'); return; } const url = URL.createObjectURL(blob); const a = document.createElement('a'); a.href = url; a.download = `倒杯快乐-${level.title}${d.sample ? '-示例' : ''}.png`; a.click(); setTimeout(() => URL.revokeObjectURL(url), 30000); toast('快乐小票已生成，浏览器将下载 PNG 图片。'); }, 'image/png');
  }
  $('#aboutButton').onclick = () => openDialog('倒杯快乐 · 设计提案', `<p>暂定名「倒杯快乐」，英文视觉标识 POUR JOY。主张是「把今天，倒成喜欢的颜色」。名称尚未进行商标与重名核查。</p><h3 class="dialog-section">这版可以体验</h3><p>4 个页面、3 道可解关卡、同色倒入、撤销、重开、灵感提示、音效开关、通关贴纸、PNG 小票导出和同题链接。进度仅在本次打开期间保留。</p><h3 class="dialog-section">正式版玩法方向</h3><p>章回关卡 → 解锁招牌饮品 → 收集手帐 → 分享同题。后续增加封签杯与订单挑战，每章只引入一种新机制。目标步数只作为可选挑战，主流程不设倒计时。</p><h3 class="dialog-section">参考与取舍</h3><p>参考 <a href="https://play.google.com/store/apps/details?id=water.sort.puzzle.game.color.sorting.free" target="_blank" rel="noreferrer">Water Sort Puzzle</a> 的倒水规则与递进挑战，以及 <a href="https://play.google.com/store/apps/details?id=com.gma.soda.sort.puzzle" target="_blank" rel="noreferrer">Soda Sort</a> 的饮料主题与无时间限制。杯子、店面、配色、文案与手帐为本提案原创。</p><p>本稿不含真实每日关卡、账号存档、排行或宿主接口；示例小票会明确标注。正式游戏资源未修改。</p>`);
  document.addEventListener('keydown', e => {
    if ($('#infoDialog').open || view !== 'play' || busy) return;
    if (e.key === 'Escape') { selected = -1; hintPair = null; renderPlay(); }
    if (/^[1-6]$/.test(e.key) && Number(e.key) <= board.length) { e.preventDefault(); selectTube(Number(e.key) - 1); }
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'z') { e.preventDefault(); undo(); }
  });
  go(challenge ? 'play' : location.hash.slice(1) || 'play', false);
  // Small read-only test surface for deterministic puzzle validation, not shown in the game.
  globalThis.PourJoyDesign = Object.freeze({ levels: levels.map(l => ({ ...l, board: clone(l.board) })), solve, moved, isWon });
})();
