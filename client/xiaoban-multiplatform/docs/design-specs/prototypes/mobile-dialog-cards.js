// Interactive review of the eight requested cards. All data and actions are local fixtures.
if (document.documentElement.dataset.edition === 'cards') {
  Object.assign(shapes, {
    sparkle: '<path d="m12 2 1.6 6.3a2.7 2.7 0 0 0 2.1 2.1L22 12l-6.3 1.6a2.7 2.7 0 0 0-2.1 2.1L12 22l-1.6-6.3a2.7 2.7 0 0 0-2.1-2.1L2 12l6.3-1.6a2.7 2.7 0 0 0 2.1-2.1L12 2Z"/><path d="M20 2v4m-2-2h4"/>',
    moon: '<path d="M20.8 13.3A8.9 8.9 0 0 1 10.7 3.2a8.9 8.9 0 1 0 10.1 10.1Z"/>',
    sun: '<circle cx="12" cy="12" r="4"/><path d="M12 2v2m0 16v2M2 12h2m16 0h2M5 5l1.5 1.5m11 11L19 19M5 19l1.5-1.5m11-11L19 5"/>',
    x: '<path d="m6 6 12 12M18 6 6 18"/>',
    comment: '<path d="M5 4h14a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H9l-5 4v-4a2 2 0 0 1-1-2V6a2 2 0 0 1 2-2Z"/><path d="M7 9h10m-10 4h6"/>',
    download: '<path d="M12 3v12m-4-4 4 4 4-4M4 16v4h16v-4"/>',
    help: '<circle cx="12" cy="12" r="9"/><path d="M9.6 8.8a2.5 2.5 0 1 1 4.4 1.8c-1.3.9-2 1.3-2 3m0 3h.01"/>',
    info: '<circle cx="12" cy="12" r="9"/><path d="M12 11v5m0-9h.01"/>',
    flag: '<path d="M5 21V4c5-4 9 4 15 0v10c-6 4-10-4-15 0"/>',
    user: '<circle cx="9.5" cy="8" r="3.4"/><path d="M3 20v-1.7a6.5 6.5 0 0 1 10-5.5m3 2.2 5 5m0-5-5 5"/>',
    shield: '<path d="m12 3 8 3v6c0 4.5-4 7-8 9-4-2-8-4.5-8-9V6l8-3Z"/><path d="M12 8v5m0 3h.01"/>',
    terminal: '<rect x="2.7" y="4.5" width="18.6" height="15" rx="3"/><path d="m6.5 9 3 3-3 3m6 0H17"/>',
    arrow: '<path d="M4 12h15m-6-6 6 6-6 6"/>',
    book: '<path d="M12 6C9 4 5 4 3 5v14c3-1 6-1 9 1 3-2 6-2 9-1V5c-2-1-6-1-9 1Zm0 0v14"/>',
    game: '<path d="M6 7h12c2 0 4 9 3 10-2 3-5-2-7-2h-4c-2 0-5 5-7 2C2 16 4 7 6 7Z"/><path d="M7 9v4m-2-2h4m6-1h.01m3 2h.01"/>',
    clock: '<circle cx="12" cy="12" r="9"/><path d="M12 6v6l4 2"/>',
    device: '<rect x="6" y="2" width="12" height="20" rx="3"/><path d="M10 5h4m-3 14h2"/>',
    layers: '<path d="m12 3 10 5-10 5L2 8l10-5Zm-9 9 9 5 9-5M3 16l9 5 9-5"/>',
  });

  const pairs = [
    { key: 'discussion', name: '评论与附件', icon: 'comment', scenes: ['comment', 'attachment'] },
    { key: 'creation', name: '求助与投稿', icon: 'flag', scenes: ['help', 'challenge'] },
    { key: 'social', name: '屏蔽与举报', icon: 'shield', scenes: ['block', 'report'] },
    { key: 'settings', name: '环境与更新', icon: 'terminal', scenes: ['tools', 'update'] },
  ];
  const scenes = {
    comment: { label: '删除评论', page: '详情页', background: 'discussion', symbol: 'comment', title: '删除这条评论？',
      action: '删除评论', result: '评论已删除', states: { ready: '待确认', error: '删除失败' } },
    attachment: { label: '购买附件', page: '帖子详情', background: 'discussion', cover: 'file', title: '离线阅读器示例.zip',
      description: '4.8 MB · 重复下载免费', action: '20 积分 · 购买并下载', result: '已购买，开始下载' },
    help: { label: '关闭求助', page: '求助详情', background: 'discussion', symbol: 'help', tile: '#705842', title: '关闭这条求助？',
      description: '怎么离线查看网页？', action: '关闭求助', result: '求助已关闭' },
    challenge: { label: '选择投稿作品', page: '官方挑战', background: 'challenge', symbol: 'flag', tile: '#5d528a', title: '选择我的作品',
      description: '仅限已上架的公开作品，每件限投一次。', action: '提交作品', result: '作品已投稿',
      states: { ready: '作品列表', empty: '暂无作品', loading: '加载中', loadError: '加载失败', stale: '挑战状态变更' } },
    block: { label: '屏蔽用户', page: '个人主页', background: 'profile', symbol: 'user', tile: '#585973', title: '屏蔽小林？',
      description: '列表和搜索将隐藏对方的帖子，回复将折叠。', action: '确认屏蔽', result: '已屏蔽小林' },
    report: { label: '举报内容', page: '帖子详情', background: 'discussion', symbol: 'flag', tile: '#695066', title: '举报这篇帖子',
      action: '提交举报', result: '举报已提交', states: { ready: '填写举报', login: '未登录', error: '提交失败' } },
    tools: { label: '移除工具环境', page: '环境配置', background: 'settings', symbol: 'terminal', tile: '#354e49', title: '移除工具环境？',
      description: '将删除本机运行时及其中安装的工具。', action: '移除环境', result: '工具环境已移除' },
    update: { label: 'App 更新', page: '关于小伴', scope: '示例版本', background: 'settings', title: '小伴 v3.2.0',
      description: '优化社区详情页的评论操作。', action: '安装更新', states: { ready: '发现新版', checking: '正在检查', latest: '已是最新版', failed: '检查失败', progress: '下载中', installError: '安装失败', system: '系统确认' } },
  };
  const reportReasons = ['垃圾广告或私发推广', '分享或交易 API Key、账号', '人身攻击或开盒', '引战或时政', '侵犯版权', '违法违规', '其他'];
  const works = [
    { title: '随身阅读', type: '工具 · v1.2.0', icon: 'book' },
    { title: '一方小院', type: '游戏 · v1.0.0', icon: 'game' },
    { title: '番茄钟', type: '已提交当前挑战', icon: 'clock', submitted: true },
  ];
  const studio = document.querySelector('#card-review');
  const pairNav = studio.querySelector('.pair-nav');
  const area = document.querySelector('#review-pair');
  const appearance = studio.querySelector('.appearance');
  studio.querySelector('.app-mark').innerHTML = icon('sparkle');
  function updateTheme(dark) {
    document.documentElement.dataset.theme = dark ? 'dark' : 'light';
    appearance.innerHTML = icon(dark ? 'sun' : 'moon');
    appearance.setAttribute('aria-label', dark ? '切换到浅色' : '切换到深色');
    appearance.title = appearance.getAttribute('aria-label');
  }
  appearance.onclick = () => updateTheme(document.documentElement.dataset.theme !== 'dark');
  updateTheme(false);

  function note(text, error = false) {
    return `<div class="card-note${error ? ' error' : ''}"${error ? ' role="alert"' : ''}>${icon('info')}<span>${text}</span></div>`;
  }
  function cover(type) {
    const art = type === 'file' ? '<div class="folder"><i class="folder-tab"></i><i class="folder-sheet"></i><span class="folder-front">ZIP</span></div>' : `<div class="release-icon">${icon('sparkle')}</div>`;
    return `<div class="card-cover cover-${type}" aria-hidden="true">${art}</div>`;
  }
  function intro(scene, key, title, description, hero) {
    const copy = description ? `<p class="card-copy">${description}</p>` : '';
    const heading = `<h3 id="title-${key}" tabindex="-1">${title}</h3>`;
    if (hero) return heading + copy;
    return `<div class="card-item-head"><div class="object-tile" style="--tile:${scene.tile || '#293036'}">${icon(scene.symbol || 'device')}</div><div>${heading}${copy}</div></div>`;
  }
  function choices(key, items, selected) {
    return `<div class="choice-list ${key === 'report' ? 'reason-list' : ''}" role="radiogroup" aria-label="${key === 'report' ? '举报原因' : '我的作品'}">${items.map((item, index) =>
      `<label class="choice-row"><input type="radio" name="${key}-choice" value="${index}" ${selected === index ? 'checked' : ''} ${item.submitted ? 'disabled' : ''}>
       ${item.icon ? `<span class="work-tile">${icon(item.icon)}</span>` : ''}<span>${item.title}${item.type ? `<small>${item.type}</small>` : ''}</span><i class="radio-dot" aria-hidden="true"></i></label>`).join('')}</div>`;
  }
  function updateCard(mode, percent) {
    const checks = {
      checking: ['正在检查更新…', ''], latest: ['已是最新版本', 'v3.1.8'], failed: ['检查更新失败', ''],
    };
    if (checks[mode]) {
      const [title, description] = checks[mode];
      return { title, description, extra: mode === 'failed' ? note('请稍后重试。', true) : '', secondary: '关闭', action: '' };
    }
    const progress = `<div class="progress-box"><div class="update-progress"><span>正在下载并校验安装包</span><span>${percent}%</span></div><progress max="100" value="${percent}" aria-label="安装包下载进度"></progress></div>`;
    const messages = {
      ready: `<p class="version-note">${icon('shield')}<span>更新保留现有数据，下载后由系统确认安装。</span></p>`,
      progress, system: note('请在 Android 系统安装界面确认更新。'),
      installError: note('下载或启动安装失败，请检查网络和剩余空间后重试。', true),
    };
    return { hero: 'version', description: mode === 'ready' ? scenes.update.description : '', extra: messages[mode], secondary: '稍后', action: mode === 'system' || mode === 'progress' ? '' : '安装更新' };
  }
  function cardContent(key, demo) {
    const scene = scenes[key];
    const { mode, selected } = demo;
    const data = { title: scene.title, description: scene.description, hero: scene.cover, action: scene.action, secondary: '取消', extra: '', disabled: false };
    switch (key) {
      case 'comment':
        data.extra = '<blockquote class="card-quote">这个方案在手机上也能运行吗？</blockquote>';
        if (mode === 'error') data.extra += note('评论删除失败，请重试。', true);
        break;
      case 'help':
        data.extra = note('尚未发放的悬赏积分将退还。');
        break;
      case 'block':
        data.extra = note('双方将取消关注，对方不会收到通知。');
        break;
      case 'challenge': {
        if (mode === 'empty') data.extra = '<p class="state-message">暂无可投稿的作品，请先从「我的作品」发布。</p>';
        else if (mode === 'loading') data.extra = '<p class="state-message" role="status">正在读取我的作品…</p>';
        else if (mode === 'loadError') data.extra = note('暂时无法读取作品列表。', true) + '<div class="state-message"><button data-do="retry">重试</button></div>';
        else data.extra = choices(key, works, selected);
        data.disabled = mode !== 'ready' || selected === null;
        if (mode === 'stale') {
          data.secondary = '关闭并刷新状态';
          data.extra += note('挑战状态已变更。', true);
        }
        break;
      }
      case 'report':
        if (mode === 'login') {
          data.description = '登录后可举报。';
          data.action = '登录';
        } else {
          data.extra = choices(key, reportReasons.map(title => ({ title })), selected) +
            '<div class="report-form"><label class="field-label" for="report-detail">补充说明（可选）<span id="char-count"></span></label><textarea id="report-detail" maxlength="500"></textarea></div>';
          data.disabled = selected === null;
          if (mode === 'error') data.extra += note('举报提交失败，请重试。', true);
        }
        break;
      case 'update':
        Object.assign(data, updateCard(mode, demo.percent));
        break;
    }
    if (demo.busy) {
      data.action = { comment: '删除中…', help: '关闭中…', challenge: '投稿中…', report: '提交中…' }[key];
      data.disabled = true;
    }
    return data;
  }
  function modal(key, demo) {
    const scene = scenes[key];
    const { title, description, action, secondary, hero, extra, disabled } = cardContent(key, demo);
    const lock = demo.busy ? 'disabled' : '';
    const trailing = key === 'attachment' ? icon('download') : key === 'update' && demo.mode === 'ready' ? icon('arrow') : '';
    return `<div class="scene-layer"><div class="scene-scrim" data-do="close"></div>
      <section class="scene-card ${hero ? 'has-cover' : ''}" role="dialog" aria-labelledby="title-${key}">
        <button class="card-close" data-do="close" aria-label="关闭${scene.label}弹窗" ${lock}>${icon('x')}</button>
        <div class="card-scroll">${hero ? cover(hero) : ''}<div class="card-pad">
          ${intro(scene, key, title, description, hero)}${extra}</div></div>
        <footer class="card-footer">${action ? `<button class="main-action" data-do="confirm" ${disabled ? 'disabled' : ''}>${action}${trailing}</button>` : ''}
          <button class="alt-action" data-do="close" ${lock}>${secondary}</button></footer>
      </section></div>`;
  }
  function board(key) {
    const scene = scenes[key];
    const wrapper = document.createElement('article');
    wrapper.className = 'scene';
    wrapper.dataset.scene = key;
    const options = Object.entries(scene.states || {}).map(([state, label]) => `<option value="${state}">${label}</option>`).join('');
    wrapper.innerHTML = `<header class="scene-heading"><h2>${scene.label}</h2><span class="scene-path">${scene.scope || scene.page}</span>
      ${options ? `<label class="scene-state"><select aria-label="${scene.label}演示状态">${options}</select>${icon('down')}</label>` : ''}</header>
      <div class="mock-phone"><div class="mock-page">${header(scene.page)}${contentPage(key, scene, 0)}
      <button class="scene-reopen" data-do="open">${key === 'update' ? '检查更新' : scene.label}</button></div>
      <div class="card-slot"></div><div class="gesture"></div><div class="toast" role="status" hidden></div></div>`;
    const slot = wrapper.querySelector('.card-slot');
    const bg = wrapper.querySelector('.mock-page');
    const trigger = wrapper.querySelector('.scene-reopen');
    const picker = wrapper.querySelector('select');
    const toast = wrapper.querySelector('.toast');
    const demo = { mode: 'ready', selected: null, detail: '', percent: 62, busy: false, open: true };
    let timer;
    let toastTimer;

    function render() {
      bg.inert = demo.open;
      slot.innerHTML = demo.open ? modal(key, demo) : '';
      if (picker) {
        picker.value = demo.mode;
        picker.disabled = demo.busy;
      }
      const field = slot.querySelector('textarea');
      if (field) {
        field.value = demo.detail;
        field.disabled = demo.busy;
        slot.querySelector('#char-count').textContent = `${demo.detail.length} / 500`;
      }
      if (demo.busy) slot.querySelectorAll('input').forEach(input => { input.disabled = true; });
    }
    function close(message = '') {
      demo.open = false;
      render();
      toast.textContent = message;
      toast.hidden = !message;
      clearTimeout(toastTimer);
      if (message) toastTimer = setTimeout(() => { toast.hidden = true; }, 2200);
      trigger.focus({ preventScroll: true });
    }
    function changeMode(mode) {
      clearTimeout(timer);
      demo.mode = mode;
      demo.busy = false;
      demo.open = true;
      demo.selected = mode === 'stale' || (key === 'report' && mode === 'error') ? 0 : null;
      demo.detail = '';
      demo.percent = 62;
      toast.hidden = true;
      render();
    }
    function confirm() {
      if (key === 'update') {
        demo.mode = 'progress';
        demo.percent = 0;
        function advance() {
          demo.percent += 25;
          if (demo.percent >= 100) demo.mode = 'system';
          render();
          if (demo.mode === 'progress') timer = setTimeout(advance, 500);
        }
        render();
        timer = setTimeout(advance, 500);
      } else if (key === 'report' && demo.mode === 'login') {
        close('演示：前往账号登录');
      } else if (['comment','help','challenge','report'].includes(key)) {
        demo.busy = true;
        if (demo.mode === 'error') demo.mode = 'ready';
        render();
        timer = setTimeout(() => { demo.busy = false; close(scene.result); }, 650);
      } else close(scene.result);
    }
    picker?.addEventListener('change', () => changeMode(picker.value));
    wrapper.addEventListener('click', event => {
      const control = event.target.closest('[data-do]');
      if (!control || control.disabled || demo.busy) return;
      switch (control.dataset.do) {
        case 'open':
          if (key === 'update' && demo.mode === 'progress') { demo.open = true; render(); }
          else changeMode('ready');
          slot.querySelector('h3').focus({ preventScroll: true });
          break;
        case 'close':
          close(demo.mode === 'stale' ? '挑战状态已刷新' : '');
          break;
        case 'retry': changeMode('ready'); break;
        case 'confirm': confirm(); break;
      }
    });
    wrapper.addEventListener('change', event => {
      if (!event.target.matches('input[type="radio"]')) return;
      demo.selected = Number(event.target.value);
      slot.querySelector('.main-action').disabled = cardContent(key, demo).disabled;
    });
    wrapper.addEventListener('input', event => {
      if (!event.target.matches('textarea')) return;
      demo.detail = event.target.value;
      slot.querySelector('#char-count').textContent = `${demo.detail.length} / 500`;
    });
    wrapper.addEventListener('keydown', event => {
      if (event.key === 'Escape' && demo.open && !demo.busy && slot.contains(event.target)) close();
    });
    render();
    return { element: wrapper, dispose() { clearTimeout(timer); clearTimeout(toastTimer); } };
  }

  let mounted = [];
  function showPair() {
    mounted.forEach(view => view.dispose());
    const pair = pairs.find(item => '#' + item.key === location.hash) || pairs[0];
    pairNav.innerHTML = pairs.map(item => `<a href="#${item.key}" ${item === pair ? 'aria-current="page"' : ''}>${icon(item.icon)}${item.name}</a>`).join('');
    mounted = pair.scenes.map(board);
    area.replaceChildren(...mounted.map(view => view.element));
  }
  window.addEventListener('hashchange', showPair);
  showPair();
}
