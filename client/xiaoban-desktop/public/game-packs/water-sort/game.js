(() => {
  'use strict';
  const R = globalThis.XiabanWaterSort, P = globalThis.PourJoyProgress, H = globalThis.PourJoyHost, A = globalThis.PourJoyArt, levels = globalThis.PourJoyLevels;
  const training = globalThis.PourJoyTraining, audio = globalThis.PourJoyAudio, motion = globalThis.PourJoyMotion;
  const $ = (s, root = document) => root.querySelector(s), $$ = s => [...document.querySelectorAll(s)];
  const colors = [{ name:'草莓', symbol:'莓', color:'#ff527a' },{ name:'抹茶', symbol:'茶', color:'#73ca38' },{ name:'芒果', symbol:'芒', color:'#ffbe22' },{ name:'香芋', symbol:'芋', color:'#a66bf0' },{ name:'蓝莓', symbol:'蓝', color:'#43b8ed' },{ name:'蜜桃', symbol:'桃', color:'#ff8b45' }];
  const chapters = [{name:'街角初遇',kind:'经典',tip:'同色相接，空杯中转。',color:'#ff9462'},{name:'果园来信',kind:'订单',tip:'带标签的杯子，最后要装满指定口味。',color:'#94cf55'},{name:'秘密配方',kind:'封签',tip:'先装满指定口味，解开封签杯。',color:'#b58aea'},{name:'金牌调饮',kind:'组合',tip:'解开封签，再按订单装杯。',color:'#ffbf47'}];
  chapters.push({name:'纯色进阶',kind:'进阶',tip:'杯子更多，试着留一个空杯备用。',color:'#7fcfd0'});
  const screen = $('#screen'), dialog = $('#infoDialog');
  let data = P.fresh(), session = null, level = levels[0], state = R.initial(level), undoStates = [], selected = -1;
  let view = 'home', loading = true, readOnly = false, busy = false, pendingView = null, hints = null, hintToken = 0, witness = new Map();
  let chapterTab = 0, journalTab = 0, roundResult = null, lastResult = null, hidden = false, toastTimer, reported = false;
  let activePour = null;
  const queuedCups = [];
  const toast = message => { clearTimeout(toastTimer); $('#toast').textContent = message; $('#toast').classList.add('show'); toastTimer = setTimeout(() => $('#toast').classList.remove('show'), 2500); };
  const won = () => session && R.isWon(state, level);
  const count = () => Object.keys(data.records).length;
  const score = () => R.completed(state) * 100;
  const starsText = n => '★'.repeat(n) + '☆'.repeat(3 - n);
  const syncAudio = () => audio.configure({ effects: data.sound, music: data.music, active: !hidden && !loading });
  const hasExtra = () => session?.extraAt != null;
  const canUndo = () => undoStates.length > 0 || hasExtra();
  function save() { if (loading || readOnly) return; data.session = session; H.save(data); }
  function storageStatus() { const b = $('#saveStatus'); b.hidden = !H.error && !readOnly; b.textContent = readOnly ? '○' : '!'; }
  globalThis.addEventListener('joy-storage-error', storageStatus);
  globalThis.addEventListener('joy-storage-restored', storageStatus);
  $('#saveStatus').onclick = () => openDialog('存档状态', `<p>${readOnly ? '当前为不保存试玩。重新打开可恢复读取本机存档。' : '保存暂未成功，进度仍在当前页面。请重试后再退出。'}</p>${readOnly ? '' : '<button class="primary" id="retrySave">重试保存</button>'}`, () => { if ($('#retrySave')) $('#retrySave').onclick = async () => { await H.save(data); storageStatus(); if (!H.error) { closeDialog(); toast('已保存'); } }; });
  let returnFocus;
  function openDialog(title, html, bind) { returnFocus = document.activeElement; $('#dialogTitle').textContent = title; $('#dialogContent').innerHTML = html; dialog.showModal(); syncInsets(); bind?.(); }
  function closeDialog() { dialog.close(); if (returnFocus?.isConnected) returnFocus.focus({ preventScroll:true }); }
  $('.dialog-close').onclick = closeDialog;
  dialog.addEventListener('click', e => { if (e.target === dialog) closeDialog(); });
  function navigate(next, push = true) {
    if (loading) return;
    if (busy) { pendingView = [next, push]; queuedCups.length = 0; return; }
    hintToken++; hints = null; selected = -1;
    if (dialog.open) closeDialog();
    view = ['home','play','levels','journal','receipt','poster'].includes(next) ? next : 'home';
    if (view === 'play' && !session) { startMain(); return; }
    if (['receipt','poster'].includes(view) && !lastResult) view = 'home';
    if (push && location.hash !== '#' + view) history.pushState({view}, '', '#' + view);
    render(); screen.scrollTop = 0;
    if (view === 'play' && session && !won() && !reported) { H.game('start'); H.game('progress', score()); reported = true; }
  }
  document.addEventListener('click', e => {
    const button = e.target.closest('[data-view]');
    if (button) navigate(button.dataset.view);
  });
  document.addEventListener('click', () => audio.unlock(), { capture: true });
  document.addEventListener('keydown', () => audio.unlock());
  globalThis.addEventListener('popstate', () => navigate(location.hash.slice(1) || 'home', false));
  function prepareWitness() {
    witness = new Map(); let s = R.initial(level);
    for (const move of level.solution) { witness.set(R.key(s), move); s = R.pour(s, level, ...move); }
  }
  function start(id, mode = 'campaign', confirm = true) {
    if (busy) return;
    const next = P.getLevel(id); if (!next) return toast('挑战码无效');
    if (mode === 'campaign' && levels.indexOf(next) > P.unlocked(data)) return toast('先完成前一关');
    if (confirm && (session?.moves.length || hasExtra()) && !won()) {
      openDialog('换一杯？', '<p>当前这杯会重新开始，已通关记录会保留。</p><button class="primary" id="switchRound">换一杯</button><button class="secondary" id="stayRound">继续当前关卡</button>', () => {
        $('#switchRound').onclick = () => { closeDialog(); start(id, mode, false); };
        $('#stayRound').onclick = closeDialog;
      }); return;
    }
    hintToken++;
    queuedCups.length = 0;
    level = next;
    chapterTab = level.chapter;
    session = { id, mode, moves: [], hints: 0, extraAt: null };
    state = R.initial(level);
    undoStates = [];
    selected = -1;
    hints = null;
    reported = false;
    roundResult = null;
    prepareWitness(); save(); navigate('play');
    if (Object.keys(level.seals).length || Object.keys(level.orders).length) toast(chapters[level.chapter].tip);
  }
  function render() { ({home:renderHome,play:renderPlay,levels:renderLevels,journal:renderJournal,receipt:renderReceipt,poster:renderPoster})[view](); syncInsets(); }
  function startMain() {
    if (data.trainingDone) start(levels[P.unlocked(data)].id, 'campaign', false);
    else {
      const next = session?.mode === 'training' && won() ? training[training.indexOf(level) + 1] : training[0];
      start(next.id, 'training', false);
    }
  }
  function completeTraining() {
    data.trainingDone = true;
    startMain();
  }
  function soundIcon() { return A.icon(data.sound || data.music ? 'sound' : 'mute'); }
  function openSound() {
    openDialog('声音设置', `<div class="sound-settings">
      <label><span>背景音乐</span><input id="musicSetting" type="checkbox" ${data.music ? 'checked' : ''}></label>
      <label><span>操作音效</span><input id="effectsSetting" type="checkbox" ${data.sound ? 'checked' : ''}></label>
    </div>`, () => {
      function update() {
        data.music = $('#musicSetting').checked;
        data.sound = $('#effectsSetting').checked;
        syncAudio();
        audio.unlock();
        audio.play('select');
        save();
        const button = $('#soundButton');
        if (button) button.innerHTML = soundIcon();
      }
      $('#musicSetting').onchange = update;
      $('#effectsSetting').onchange = update;
    });
  }
  function toolButton(id, icon, label, disabled = false) {
    return `<button class="tool" id="${id}" ${disabled ? 'disabled' : ''}><span>${A.icon(icon)}</span>${label}</button>`;
  }

  function renderPlay() {
    if (won()) {
      renderResult();
      return;
    }
    const focusTube = document.activeElement?.dataset?.tube;
    const practicing = session.mode === 'training';
    const modeLabel = practicing ? `练习 ${training.indexOf(level) + 1} / ${training.length}`
      : session.mode === 'campaign' ? `第 ${levels.indexOf(level) + 1} 关`
      : session.mode === 'daily' ? '每日特调' : '好友挑战';
    const orderIcons = colors.slice(0, level.colors).map((c, i) => {
      const done = state.board.some((tube, index) => tube[0] === i && cupComplete(tube, index));
      return `<span class="order-dot ${done ? 'done' : ''}" style="--flavor:${c.color}" title="${c.name}"><i></i>${c.symbol}</span>`;
    }).join('');
    const heading = practicing ? '调饮练习' : chapters[level.chapter].name;
    screen.innerHTML = `<section class="game-page ${state.board.length > 6 ? 'many-cups' : ''}">
      <div class="game-topline"><button class="chapter-tag" data-view="levels"><i></i>${heading}⌄</button>
        <button class="icon-button" id="soundButton" aria-label="声音设置">${soundIcon()}</button></div>
      <div class="level-heading"><div><h2>${level.name}</h2><p>${modeLabel}</p></div>
        <div class="step-count"><strong id="moveCount">${String(session.moves.length).padStart(2, '0')}</strong><span>步</span></div></div>
      <div class="order-strip"><span class="completion-count">${state.board.filter(cupComplete).length}<small> / ${level.colors}</small></span>
        <div class="order-icons">${orderIcons}</div></div>
      <div class="board-area"><div class="board" role="group" aria-label="调饮杯盘"></div></div>
      <p class="game-tip" aria-live="polite"></p>
      <div class="tools">
        ${toolButton('undo', 'undo', '撤销', !canUndo())}
        ${toolButton('hint', 'hint', '灵感')}
        ${practicing ? '' : toolButton('addCup', 'addCup', '加空杯', hasExtra())}
        ${toolButton('reset', 'reset', '重调')}
      </div>
      <div class="game-bottom"><button data-view="home">小店</button>
        ${practicing ? '<button id="skipTraining">跳过练习</button>' : `<button id="starHelp" title="星级规则">${level.par} 步三星</button>`}
        <button id="rules">玩法 ⓘ</button></div>
      </section>`;
    renderBoard();
    if (focusTube !== undefined) $(`[data-tube="${focusTube}"]`)?.focus({ preventScroll: true });
    $('#soundButton').onclick = openSound;
    $('#undo').onclick = undo;
    $('#hint').onclick = showHint;
    $('#reset').onclick = () => start(level.id, session.mode);
    $('#rules').onclick = showRules;
    if (practicing) $('#skipTraining').onclick = completeTraining;
    else {
      $('#addCup').onclick = addCup;
      $('#starHelp').onclick = () => openDialog('星级挑战', `<p>★ 完成关卡<br>★★ 不超过 ${level.par + 5} 步<br>★★★ 不超过 ${level.par} 步，且未用灵感或空杯辅助</p><p>撤销不限次数，超出目标也能过关。</p>`);
    }
  }

  function cupComplete(tube, i) {
    if (!R.full(tube)) return false;
    const color = tube[0];
    const order = level.orders[i];
    return order === undefined ? !Object.values(level.orders).includes(color) : order === color;
  }

  function orderInstruction() {
    const pending = Object.entries(level.orders).filter(([index]) => !cupComplete(state.board[index], Number(index)));
    for (const [index, color] of pending) {
      const to = Number(index);
      const from = state.board.findIndex(tube => tube[0] === color && R.full(tube));
      if ((selected < 0 || selected === from) && R.getPour(state, level, from, to).valid) {
        return `把 ${from + 1} 号杯的${colors[color].name}倒进 ${to + 1} 号订单杯`;
      }
    }
    if (selected < 0 && pending.length) {
      const [index, color] = pending[0];
      return `${Number(index) + 1} 号订单杯需要装满${colors[color].name}`;
    }
    return '';
  }

  function renderBoard() {
    const board = $('.board');
    board.dataset.count = state.board.length;
    board.style.setProperty('--columns', Math.ceil(state.board.length / 2));
    board.innerHTML = state.board.map((tube, i) => {
      const sealed = R.locked(state, level, i);
      const order = level.orders[i];
      const complete = cupComplete(tube, i);
      const content = sealed ? '封签，先完成' + colors[level.seals[i]].name
        : tube.length ? '从底到顶：' + tube.map(c => colors[c].name).join('、') : '空杯';
      const label = `${i + 1}号杯，${content}${order !== undefined ? '，订单：' + colors[order].name : ''}${complete ? '，已完成' : ''}`;
      return `<button class="tube-button ${complete ? 'complete' : ''} ${sealed ? 'sealed' : ''}" data-tube="${i}" aria-label="${label}">
        <span class="tube-shadow"></span><span class="glass"><span class="liquid-stack">
          ${tube.map(c => `<span class="liquid" style="--flavor:${colors[c].color}">${colors[c].symbol}</span>`).join('')}
        </span>
          ${sealed ? `<span class="seal-band" style="--flavor:${colors[level.seals[i]].color}">先调${colors[level.seals[i]].symbol}<b>⌑</b></span>` : complete ? '<span class="complete-seal">✓</span>' : ''}
        </span>
        <span class="cup-caption">${order !== undefined ? `<span class="order-badge" style="--flavor:${colors[order].color}">${colors[order].symbol} · 订单</span>` : String(i + 1).padStart(2, '0')}</span>
      </button>`;
    }).join('');
    $$('[data-tube]').forEach(button => { button.onclick = () => choose(Number(button.dataset.tube)); });
    updateSelection();
  }

  function updateSelection(message) {
    const firstMove = level === training[0] && session.moves.length === 0;
    const [firstSource, firstTarget] = training[0].solution[0];
    const cue = firstMove ? (selected < 0 ? firstSource : selected === firstSource ? firstTarget : -1) : -1;
    let hasTarget = false;
    $$('[data-tube]').forEach(button => {
      const i = Number(button.dataset.tube);
      const target = selected >= 0 && R.getPour(state, level, selected, i).valid;
      hasTarget ||= target;
      button.classList.toggle('selected', i === selected);
      button.classList.toggle('target', target);
      button.classList.toggle('hint', hints?.includes(i) === true);
      button.classList.toggle('training-cue', cue === i);
      button.setAttribute('aria-pressed', String(i === selected));
    });
    const instruction = orderInstruction() || (selected >= 0 ? (hasTarget ? '点亮边的杯子倒入' : '这杯暂时倒不出，试试换一杯')
      : firstMove ? '先点有光圈的杯子' : session.moves.length ? '' : chapters[level.chapter].tip);
    $('.game-tip').textContent = message ?? instruction;
  }

  function rejectCup(i, message) {
    audio.play('invalid');
    const button = $(`[data-tube="${i}"]`);
    button.classList.remove('shake');
    void button.offsetWidth;
    button.classList.add('shake');
    updateSelection(message);
  }

  async function choose(i) {
    if (won() || hidden) return;
    if (busy) {
      if (!pendingView && queuedCups.length < 2 && queuedCups.at(-1) !== i) queuedCups.push(i);
      return;
    }
    hintToken++;
    hints = null;
    $('#hint').disabled = false;
    if (R.locked(state, level, i)) return rejectCup(i, '先装满一杯' + colors[level.seals[i]].name);
    if (selected === i) {
      selected = -1;
      updateSelection();
      return;
    }
    if (selected < 0) {
      if (!state.board[i].length) return rejectCup(i, '先选有饮料的杯子');
      selected = i;
      audio.play('select');
      updateSelection();
      return;
    }

    const from = selected;
    const info = R.getPour(state, level, from, i);
    if (!info.valid) {
      return rejectCup(i, info.reason === 'full' ? '这杯满了，换个接收杯' : '只能倒进空杯或同色杯');
    }
    if (session.moves.length >= 4096) return toast('步数已满，请撤销或重调');
    busy = true;
    $$('.game-page button:not([data-tube])').forEach(button => { button.disabled = true; });
    undoStates.push(state);
    const wasOpened = state.opened.length;
    state = R.pour(state, level, from, i);
    session.moves.push([from, i]);
    selected = -1;
    const finished = won();
    if (finished) roundResult = P.finish(data, session);

    // Commit before animating so leaving the page never stores a half-pour.
    save();
    H.game('progress', score());
    if (finished) {
      H.game('result', score());
      reported = false;
    }
    H.haptic();
    try {
      if (matchMedia('(prefers-reduced-motion: reduce)').matches) {
        audio.play('pour', .12);
      } else {
        activePour = motion.pour($(`[data-tube="${from}"] .glass`), $(`[data-tube="${i}"] .glass`), {
          amount: info.amount,
          color: colors[info.color].color,
          onFlow: duration => audio.play('pour', duration),
        });
        await activePour.finished;
      }
    } catch (error) {
      console.warn('倒水动画未完成', error);
    } finally {
      activePour = null;
      busy = false;
    }
    if (pendingView) {
      const [next, push] = pendingView;
      pendingView = null;
      navigate(next, push);
      return;
    }
    renderPlay();
    if (finished) {
      queuedCups.length = 0;
      audio.play('win');
    } else {
      if (state.opened.length > wasOpened) toast('封签已解开');
      if (cupComplete(state.board[i], i)) {
        $(`[data-tube="${i}"]`).classList.add('just-completed');
        audio.play('complete');
      }
      while (queuedCups.length && !busy && !hidden) void choose(queuedCups.shift());
    }
  }

  function addCup() {
    if (busy || won() || hasExtra() || session.mode === 'training') return;
    hintToken++;
    hints = null;
    selected = -1;
    session.extraAt = session.moves.length;
    state = { ...state, board: [...state.board, []] };
    save();
    audio.play('cup');
    renderPlay();
    $('.game-tip').textContent = '多了一只空杯 · 本关最多两星';
  }

  function undo() {
    if (busy || won() || !canUndo()) return;
    hintToken++;
    if (session.moves.length === session.extraAt) {
      state = { ...state, board: state.board.slice(0, -1) };
      session.extraAt = null;
    } else {
      state = undoStates.pop();
      session.moves.pop();
    }
    selected = -1;
    hints = null;
    save();
    audio.play('undo');
    H.game('progress', score());
    renderPlay();
  }
  async function showHint() {
    if (busy || won()) return;
    const token=++hintToken, startKey=R.key(state); let move=witness.get(startKey);
    if (!move) {
      const search=R.createSearch(state,level,30000,true), started=performance.now();
      $('#hint').disabled=true; $('.game-tip').textContent='找一点灵感…';
      while (token===hintToken && view==='play') {
        const result=search.step(100);
        if (result.done) { move=result.path?.[0]; break; }
        if (performance.now()-started>1800) break;
        await new Promise(resolve=>setTimeout(resolve,0));
      }
    }
    if (token!==hintToken || view!=='play') return;
    $('#hint').disabled = false;
    if (!move) {
      updateSelection(session.mode === 'training' || hasExtra() ? '试试撤销或重调' : '暂未找到解法，可以撤销或加空杯');
      return;
    }
    session.hints = Math.min(4096, session.hints + 1);
    hints = move;
    selected = move[0];
    save();
    updateSelection(`${move[0] + 1} 号杯 → ${move[1] + 1} 号杯`);
  }
  function renderResult() {
    const result = roundResult;
    const index = levels.indexOf(level);
    const campaign = session.mode === 'campaign';
    const practicing = session.mode === 'training';
    const next = campaign ? levels[index + 1] : practicing ? training[training.indexOf(level) + 1] : null;
    const label = campaign ? `第 ${index + 1} 关完成`
      : practicing ? `${level.name} · 完成` : `${session.mode === 'daily' ? '每日特调' : '好友挑战'}完成`;
    const action = practicing ? (next ? '下一项练习' : '开始正式调饮')
      : next ? `下一关 · 第 ${index + 2} 关` : '返回选关';
    screen.innerHTML = `<section class="result-page" aria-labelledby="roundTitle">
      <div class="round-result">
        ${practicing ? '<div class="win-badge" aria-hidden="true">✓</div>' : `<div class="win-badge" aria-label="获得 ${result.stars} 星">${starsText(result.stars)}</div>`}
        <h2 id="roundTitle">完美出杯！</h2>
        <p>${label} · ${result.moves} 步${result.assisted && !practicing ? ' · 辅助调饮' : ''}</p>
        <div class="win-drink" aria-hidden="true">${A.drink(colors[Number(level.id.slice(-2)) % colors.length].color)}</div>
        <button class="primary" id="nextLevel">${action} ${A.icon('arrow')}</button>
        <div class="result-secondary">
          ${practicing ? '<button class="secondary" id="skipTraining">跳到正式关卡</button>' : '<button class="secondary" data-view="receipt" id="roundReceipt">查看小票</button>'}
          <button class="secondary" id="replayLevel">重玩本关</button>
        </div>
      </div>
    </section>`;
    $('#nextLevel').onclick = () => {
      if (next) start(next.id, session.mode, false);
      else if (practicing) completeTraining();
      else navigate('levels');
    };
    if (practicing) $('#skipTraining').onclick = completeTraining;
    else $('#roundReceipt').onclick = () => { lastResult = result; };
    $('#replayLevel').onclick = () => start(level.id, session.mode, false);
    screen.scrollTop = 0;
    $('#nextLevel').focus({ preventScroll: true });
  }
  function showRules() { openDialog('调饮规则','<ul><li>同色相接，空杯中转。连续同色一起倒，每杯最多四层。</li><li>所有非空杯装满单一口味即完成。</li><li>订单杯可暂存其他口味，通关时必须装满标签口味。</li><li>封签杯要先完成指定口味才能使用；撤销也会恢复封签。</li></ul><p>没有倒计时。数字 1–9 选杯，Esc 取消，Ctrl/Cmd+Z 撤销。</p>'); }
  function bottom(active) { return `<nav class="bottom-nav"><button data-view="home" class="${active==='home'?'active':''}">${A.icon('home')}小店</button><button data-view="levels" class="${active==='levels'?'active':''}">${A.icon('cup')}菜单</button><button data-view="journal" class="${active==='journal'?'active':''}">${A.icon('book')}手帐</button></nav>`; }
  function renderHome() {
    const resumable=session&&!won(), next=levels[P.unlocked(data)], today=P.today(), dailyRecord=data.daily[today];
    screen.innerHTML=`<section class="home-page"><div class="home-copy"><h2>来一杯<span>快乐！</span></h2></div><div class="shop-art">${A.shopSVG()}</div><div class="home-action"><button class="primary" id="continue">${resumable ? '继续调饮' : data.trainingDone ? '开始调饮' : '开始练习'} ${A.icon('arrow')}</button><div class="continue-info"><span>${count()} / ${levels.length} 已完成</span><span>${resumable ? level.name : data.trainingDone ? next.name : training[0].name}</span></div></div><button class="daily-card" id="daily"><span class="daily-mark">✳</span><span><strong>每日特调</strong><small>${today.slice(5,7)} / ${today.slice(7,9)}${dailyRecord?' · '+starsText(dailyRecord.stars):' · 每天一杯'}</small></span><span class="arrow">↗</span></button><button class="challenge-entry" id="challenge">输入好友挑战码 ↗</button>${bottom('home')}</section>`;
    $('#continue').onclick=()=>resumable ? navigate('play') : startMain();
    $('#daily').onclick=()=>start(today,'daily'); $('#challenge').onclick=challengeDialog;
  }
  function challengeDialog() {
    openDialog('好友同题','<label class="challenge-label" for="challengeCode">挑战码</label><input class="dialog-input" id="challengeCode" placeholder="PJ-C01 或 PJ-D20261112" maxlength="20" autocomplete="off" autocapitalize="characters"><p id="challengeError" role="status"></p><button class="primary" id="openChallenge">开始挑战</button>',()=>{
      $('#openChallenge').onclick=()=>{ const code=$('#challengeCode').value.trim().toLowerCase().replace(/^pj-/,''); if(!/^(c\d{2}|d\d{8})$/.test(code)||!P.getLevel(code)){ $('#challengeError').textContent='挑战码无效，请检查后重试';return; } closeDialog();start(code,'challenge'); };
      $('#challengeCode').onkeydown=e=>{if(e.key==='Enter')$('#openChallenge').click();}; $('#challengeCode').focus();
    });
  }
  function renderLevels() {
    const unlocked=P.unlocked(data), chapter=chapters[chapterTab];
    screen.innerHTML=`<section class="menu-page"><div class="page-title"><h2>调饮菜单</h2><span>${count()} / ${levels.length}</span></div><nav class="chapter-tabs" aria-label="章节">${chapters.map((c,i)=>`<button data-chapter="${i}" aria-pressed="${i===chapterTab}" class="${i===chapterTab?'active':''}" style="--chapter:${c.color}">0${i+1}<small>${c.kind}</small></button>`).join('')}</nav><div class="chapter-intro"><h3>${chapter.name}</h3><p>${chapter.tip}</p></div><div class="level-grid">${levels.slice(chapterTab*9,chapterTab*9+9).map((l,i)=>{const locked=levels.indexOf(l)>unlocked,record=data.records[l.id];return `<button data-level="${l.id}" class="level-tile ${locked?'locked':''}" ${locked?'disabled':''}><strong>${locked?'⌑':String(chapterTab*9+i+1).padStart(2,'0')}</strong><span>${l.name}</span><small>${record?starsText(record.stars):locked?'待解锁':'☆ ☆ ☆'}</small></button>`;}).join('')}</div><button class="secondary" id="practice">调饮练习</button>${bottom('levels')}</section>`;
    $('#practice').onclick = () => start(training[0].id, 'training');
    $$('[data-chapter]').forEach(b=>b.onclick=()=>{chapterTab=+b.dataset.chapter;renderLevels();}); $$('[data-level]').forEach(b=>b.onclick=()=>start(b.dataset.level));
  }
  function renderJournal() {
    const stickers=levels.filter((_,i)=>i%3===2), unlocked=stickers.filter(l=>data.records[l.id]).length;
    screen.innerHTML=`<section class="journal-page"><div class="page-title"><h2>风味手帐</h2><span>${unlocked} / ${stickers.length}</span></div><div class="collection-progress"><div class="track"><span style="width:${unlocked / stickers.length * 100}%"></span></div><span>${Object.values(data.records).reduce((sum,r)=>sum+r.stars,0)} ★</span></div><nav class="journal-tabs">${chapters.map((c,i)=>`<button data-journal="${i}" class="${journalTab===i?'active':''}">${c.name}</button>`).join('')}</nav><div class="sticker-grid">${stickers.slice(journalTab*3,journalTab*3+3).map((l,i)=>{const earned=data.records[l.id];return `<button class="sticker ${earned?'':'locked'}" data-sticker="${l.id}"><span class="tiny">NO. ${journalTab*3+i+1}</span><span class="sticker-art">${A.drink(colors[(journalTab*3+i)%6].color)}</span><h3>${l.name}</h3><p>${earned?starsText(earned.stars):'完成第 '+Number(l.id.slice(1))+' 关'}</p></button>`;}).join('')}</div>${bottom('journal')}</section>`;
    $$('[data-journal]').forEach(b=>b.onclick=()=>{journalTab=+b.dataset.journal;renderJournal();});
    $$('[data-sticker]').forEach(b=>b.onclick=()=>{const l=P.getLevel(b.dataset.sticker),record=data.records[l.id]; if(!record)return toast('完成第 '+Number(l.id.slice(1))+' 关收集'); openDialog(l.name,`<div class="win-drink">${A.drink(colors[Number(l.id.slice(1))%6].color)}</div><p class="center">${record.moves} 步 · ${starsText(record.stars)}</p><button class="primary" id="viewReceipt">查看小票</button>`,()=>{$('#viewReceipt').onclick=()=>{lastResult={...record,id:l.id,mode:'campaign'};closeDialog();navigate('receipt');};});});
  }
  function renderReceipt() {
    const r=lastResult,l=P.getLevel(r.id);
    screen.innerHTML=`<section class="share-page"><div class="share-heading"><h2>快乐小票</h2><span>${r.mode==='campaign'?'通关记录':r.mode==='daily'?'每日特调':'好友挑战'}</span></div><div class="receipt"><span class="eyebrow">POUR JOY</span><h3>今日快乐小票</h3><div class="receipt-art">${A.drink(colors[Number(l.id.slice(-2))%6].color)}<span class="stamp">快乐<br>已打包</span></div><h4 class="receipt-name">${l.name}</h4><div class="receipt-data"><div><strong>${r.moves}</strong><span>调饮步数</span></div><div><strong>${l.colors}</strong><span>风味归位</span></div><div><strong>${r.stars} ★</strong><span>本次星级</span></div></div><div class="receipt-line"><span>${r.assisted?'辅助调饮':'自由调饮'}</span><span>PJ-${l.id.toUpperCase()}</span></div><div class="barcode" aria-hidden="true"></div><div class="receipt-footer">KEEP THIS LITTLE JOY.</div></div><div class="share-actions"><button class="primary" id="saveReceipt">${A.icon('download')}${H.native&&!H.canSave()?'查看分享图':'保存小票'}</button><button class="secondary" id="shareChallenge">${H.canShare()?'分享挑战':'复制挑战码'} ↗</button></div><button class="secondary" data-view="home">回小店</button></section>`;
    $('#saveReceipt').onclick=saveReceipt;
    $('#shareChallenge').onclick=shareChallenge;
    if (r === roundResult && won()) {
      const returnButton = $('[data-view="home"]', screen);
      returnButton.dataset.view = 'play';
      returnButton.textContent = '返回结算';
    }
  }
  function poster() {
    const r=lastResult,l=P.getLevel(r.id),canvas=document.createElement('canvas');canvas.width=900;canvas.height=1200;const c=canvas.getContext('2d');
    const box=(x,y,w,h,r,color)=>{c.fillStyle=color;c.beginPath();c.roundRect(x,y,w,h,r);c.fill();};
    const text=(s,y,size,color='#8f472e',weight=600)=>{c.fillStyle=color;c.font=`${weight} ${size}px "Microsoft YaHei",sans-serif`;c.textAlign='center';c.fillText(s,450,y);};
    box(0,0,900,1200,0,'#ffd49a');box(65,60,770,1080,15,'#fffaf0');for(let x=65;x<835;x+=70)box(x,60,35,12,0,'#f36c47');
    text('POUR JOY',142,23,'#e96442');text('今日快乐小票',220,51);text(l.name,283,29,'#d56e43');
    c.save();c.translate(450,470);c.rotate(-.12);box(24,-115,13,100,5,'#f36b4f');box(-86,-62,172,220,33,colors[Number(l.id.slice(-2))%6].color);box(-86,-62,172,65,[20,20,0,0],'#fff0c9');box(-95,-72,190,22,12,'#fff7dc');box(-46,20,92,80,40,'#fff9e8');c.fillStyle='#ef6c45';c.font='bold 32px Georgia';c.textAlign='center';c.fillText('joy',0,72);c.restore();
    text(starsText(r.stars),710,40,'#efa52a');text(`${r.moves} 步 · ${l.colors} 种风味`,786,32);text(r.assisted?'辅助调饮':'自由调饮',833,21,'#b28a67');
    c.setLineDash([7,7]);c.strokeStyle='#dabd91';c.beginPath();c.moveTo(125,875);c.lineTo(775,875);c.stroke();c.setLineDash([]);
    text('同题挑战码',931,23,'#ab7b50');text('PJ-'+l.id.toUpperCase(),982,31,'#ea6b45');text('倒杯快乐 · 小店 → 输入好友挑战码',1054,21,'#a4805b');
    return canvas;
  }
  function saveReceipt() {
    if(H.native&&!H.canSave()){navigate('poster');return;}
    const canvas=poster(),name='倒杯快乐-'+lastResult.id+'.png';
    if(H.canSave()){H.saveImage(canvas.toDataURL('image/png'),name).then(()=>toast('已保存')).catch(()=>toast('未保存，可查看图片截图保存'));return;}
    canvas.toBlob(blob=>{if(!blob)return toast('生成失败，请重试');const url=URL.createObjectURL(blob),a=document.createElement('a');a.href=url;a.download=name;a.click();setTimeout(()=>URL.revokeObjectURL(url),30000);},'image/png');
  }
  function renderPoster() { screen.innerHTML='<section class="poster-page"><div class="page-title"><h2>分享图</h2><button class="secondary" data-view="receipt">返回</button></div><p>截图保存</p><img alt="通关小票与好友挑战码"></section>';$('.poster-page img').src=poster().toDataURL('image/png'); }
  async function shareChallenge() {
    const code='PJ-'+lastResult.id.toUpperCase(),text=`我用${lastResult.moves}步调好这杯！挑战码 ${code}。打开倒杯快乐，在小店输入挑战码。`;
    try {
      if(H.canShare())await H.share({title:'倒杯快乐',text:text.slice(0,60),image:poster().toDataURL('image/png'),query:{challenge:lastResult.id}});
      else{await H.copy(text);toast('挑战码已复制');}
    }catch(_){openDialog('好友挑战码',`<p>在小店输入下方挑战码，游玩同一题。</p><input class="dialog-input" id="copyCode" readonly value="${code}">`,()=>$('#copyCode').select());}
  }
  function syncInsets(){
    activePour?.finish();
    const root=document.documentElement,computed=getComputedStyle(root),num=k=>parseFloat(computed.getPropertyValue(k))||0;
    const left=innerWidth-num('--xb-capsule-right')-num('--xb-capsule-width'),top=num('--xb-capsule-top'),bottom=top+num('--xb-capsule-height');
    const header=$('.app-header'),rect=header.getBoundingClientRect();
    const overlaps=num('--xb-capsule-width')&&rect.top<bottom+8&&rect.bottom>top&&rect.right>left;
    header.classList.toggle('has-capsule',!!overlaps);
    header.style.setProperty('--capsule-clearance',overlaps?Math.min(rect.width-130,Math.max(16,rect.right-left+8))+'px':'16px');
    if(dialog.open){const r=dialog.getBoundingClientRect();dialog.classList.toggle('close-left',!!num('--xb-capsule-width')&&r.right-5>left-8&&r.right-49<innerWidth-num('--xb-capsule-right')+8&&r.top+3<bottom+8&&r.top+47>top-8);}
    root.style.setProperty('--visible-height',(globalThis.visualViewport?.height||innerHeight)+'px');
  }
  function hide() {
    hidden = true;
    hintToken++;
    queuedCups.length = 0;
    activePour?.finish();
    syncAudio();
    save();
  }
  function show() {
    hidden = false;
    syncInsets();
    syncAudio();
    if (!loading && !busy) {
      if (view === 'home') renderHome();
      else if (view === 'play') renderPlay();
    }
  }
  screen.addEventListener('scroll', () => activePour?.finish(), { passive: true });
  H.on('hide',hide);H.on('show',show);H.on('safearea',syncInsets);
  document.addEventListener('visibilitychange',()=>document.hidden?hide():show());globalThis.addEventListener('pagehide',hide);globalThis.addEventListener('resize',syncInsets);globalThis.visualViewport?.addEventListener('resize',syncInsets);
  document.addEventListener('keydown', e => {
    if (loading || dialog.open || view !== 'play' || e.target.matches?.('input,textarea')) return;
    if (/^[1-9]$/.test(e.key) && Number(e.key) <= state.board.length) {
      e.preventDefault();
      void choose(Number(e.key) - 1);
    }
    if (busy || won()) return;
    if (e.key === 'Escape') {
      selected = -1;
      hints = null;
      hintToken++;
      $('#hint').disabled = false;
      updateSelection();
    }
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'z') {
      e.preventDefault();
      undo();
    }
  });
  async function boot(temporary=false){
    loading=true;screen.innerHTML='<div class="loading" role="status">正在开店…</div>';
    try{
      let timer;const raw=temporary?null:await Promise.race([H.read(),new Promise((_,reject)=>{timer=setTimeout(()=>reject(Error('timeout')),5000);})]).finally(()=>clearTimeout(timer));
      readOnly=temporary;data=P.sanitize(raw);session=data.session;
      if(session){level=P.getLevel(session.id);const restored=P.replay(level, session.moves, session.extraAt);state=restored.state;undoStates=restored.history;prepareWitness();if(won())roundResult=P.finish(data,session);}
      loading=false;syncAudio();H.ready();storageStatus();
      const query=H.query();if(query&&P.getLevel(query)){navigate('home',false);start(query,'challenge');}else navigate(session&&location.hash==='#play'?'play':'home',false);
    }catch(_){screen.innerHTML='<div class="loading"><h2>存档暂时读不到</h2><p>已保存的进度不会被覆盖。</p><button class="primary" id="retryLoad">重试</button><button class="secondary" id="temporaryPlay">不保存试玩</button></div>';H.ready();$('#retryLoad').onclick=()=>boot();$('#temporaryPlay').onclick=()=>boot(true);}
  }
  boot();
})();
