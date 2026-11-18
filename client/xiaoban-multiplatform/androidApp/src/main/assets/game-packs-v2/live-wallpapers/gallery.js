(() => {
  const host = window.xb;
  const library = document.getElementById('library');
  const preview = document.getElementById('preview');
  const visual = preview.querySelector('.preview-visual');
  const overlays = preview.querySelector('.overlays');
  const title = document.getElementById('preview-title');
  const back = document.getElementById('back');
  const apply = document.getElementById('apply');
  const status = document.getElementById('status');
  const toast = document.getElementById('toast');
  const cards = new Map([...library.querySelectorAll('[data-scene]')].map(card => [card.dataset.scene, card]));
  const modes = [...preview.querySelectorAll('[data-mode]')];
  const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)');
  const darkTheme = matchMedia('(prefers-color-scheme: dark)');
  const timeFormat = new Intl.DateTimeFormat('zh-CN', {hour: '2-digit', minute: '2-digit', hourCycle: 'h23'});
  const dateFormat = new Intl.DateTimeFormat('zh-CN', {month: 'numeric', day: 'numeric', weekday: 'short'});

  let selection = {home: null, lock: null, switchTargets: {}};
  let supported = null;
  let selected = null;
  let mode = 'wallpaper';
  let canvas;
  let scene;
  let ready = false;
  let shown = true;
  let applying = false;
  let toastTimer;

  function setTheme(theme = host?.info?.theme) {
    document.documentElement.dataset.theme = theme || (darkTheme.matches ? 'dark' : 'light');
  }

  function setStatus(message) {
    status.textContent = message;
    status.hidden = !message;
    preview.classList.toggle('has-status', Boolean(message));
  }

  function updateApply() {
    apply.disabled = applying || !supported;
    const target = selection.switchTargets[mode];
    const switchLabel = {home: '切换桌面壁纸', lock: '切换锁屏壁纸', both: '切换桌面和锁屏壁纸'}[target];
    apply.textContent = applying ? '正在应用…' : switchLabel || '设为壁纸';
  }

  function notify(message) {
    clearTimeout(toastTimer);
    toast.textContent = message;
    toast.hidden = false;
    toastTimer = setTimeout(() => { toast.hidden = true; }, 2400);
  }

  function updateSelection(current) {
    selection = {home: current.home, lock: current.lock, switchTargets: current.switchTargets || {}};
    for (const [id, card] of cards) {
      const onHome = selection.home === id;
      const onLock = selection.lock === id;
      let label = '';
      if (onHome && onLock) label = '桌面和锁屏使用中';
      else if (onHome) label = '桌面使用中';
      else if (onLock) label = '锁屏使用中';
      const badge = card.querySelector('.applied-badge');
      badge.textContent = label;
      badge.hidden = !label;
      const name = card.querySelector('strong').textContent;
      card.setAttribute('aria-label', ['预览' + name, label].filter(Boolean).join('，'));
    }
  }

  async function refreshStatus() {
    if (!ready || applying) return;
    try {
      const current = await host.liveWallpapers.status();
      if (applying) return;
      if (current.supported && !supported) setStatus('');
      supported = current.supported;
      updateSelection(current);
      if (!supported) setStatus('此设备当前无法应用动态壁纸。');
    } catch (_) {
      supported = false;
      setStatus('暂时无法读取系统壁纸状态，请重新打开再试。');
    }
    updateApply();
  }

  function updateMotion() {
    scene?.setActive(shown && !document.hidden && !reducedMotion.matches);
    if (canvas) canvas.hidden = reducedMotion.matches;
  }

  function showMode(next) {
    mode = next;
    updateApply();
    for (const button of modes) button.setAttribute('aria-pressed', String(button.dataset.mode === mode));
    overlays.replaceChildren();
    const template = document.getElementById(mode + '-overlay');
    if (template) {
      overlays.append(template.content.cloneNode(true));
      const now = new Date();
      overlays.querySelector('[data-time]').textContent = timeFormat.format(now);
      overlays.querySelector('[data-date]').textContent = dateFormat.format(now);
    }
  }

  function showScene(id) {
    const card = cards.get(id);
    const previous = selected;
    scene?.destroy();
    scene = undefined;
    canvas?.remove();
    canvas = undefined;
    selected = card ? id : null;
    library.hidden = Boolean(card);
    preview.hidden = !card;
    host?.liveWallpapers.previewing(Boolean(card)).catch(() => {});

    if (!card) {
      cards.get(previous)?.focus({preventScroll: true});
      return;
    }

    toast.hidden = true;
    title.textContent = card.querySelector('strong').textContent;
    preview.style.setProperty('--poster', 'url("' + card.querySelector('.poster').src + '")');
    if (!host) setStatus('在小伴 Android App 内可设为壁纸。');
    else if (supported === false) setStatus('此设备当前无法应用动态壁纸。');
    else if (supported === null) setStatus('正在检查系统壁纸设置…');
    else setStatus('');

    let initialMode = 'wallpaper';
    if (selection.switchTargets.wallpaper !== 'both') {
      if (selection.home === id) initialMode = 'home';
      else if (selection.lock === id) initialMode = 'lock';
    }
    showMode(initialMode);

    canvas = document.createElement('canvas');
    visual.prepend(canvas);
    try {
      scene = window.createWallpaperScene(canvas, id, 30);
      canvas.addEventListener('pointerup', event => {
        if (mode === 'lock') return;
        const bounds = canvas.getBoundingClientRect();
        scene.tap((event.clientX - bounds.left) / bounds.width, (event.clientY - bounds.top) / bounds.height);
      });
    } catch (_) {
      canvas.remove();
      canvas = undefined;
    }
    updateMotion();
    back.focus({preventScroll: true});
  }

  for (const [id, card] of cards) card.addEventListener('click', () => {
    history.pushState({wallpaper: id}, '');
    showScene(id);
  });
  for (const button of modes) button.addEventListener('click', () => showMode(button.dataset.mode));
  back.addEventListener('click', () => history.back());
  window.addEventListener('popstate', event => showScene(event.state?.wallpaper));
  document.addEventListener('visibilitychange', updateMotion);
  reducedMotion.addEventListener('change', updateMotion);
  darkTheme.addEventListener('change', () => setTheme());
  setTheme();

  apply.addEventListener('click', async () => {
    if (applying || !supported || !selected) return;
    const requested = selected;
    applying = true;
    updateApply();
    setStatus('');
    try {
      const result = await host.liveWallpapers.preview(requested, mode);
      updateSelection(result);
      if (requested !== selected) return;
      if (result.outcome === 'applied') {
        history.back();
        notify('壁纸已应用');
      } else if (result.outcome === 'unverified') {
        setStatus('系统已确认，暂时无法核实壁纸的使用位置。');
      } else {
        setStatus('没有检测到新的小伴壁纸。请在系统壁纸预览中确认应用。');
      }
    } catch (error) {
      if (error.code === 'gesture') setStatus('请再点一次按钮。');
      else if (error.code === 'save_failed') setStatus('未能保存壁纸，请重试。');
      else setStatus('未能打开系统动态壁纸预览，请重试。');
    } finally {
      applying = false;
      updateApply();
    }
  });

  if (host) {
    host.on('theme', setTheme);
    host.on('show', () => {
      shown = true;
      updateMotion();
      refreshStatus();
    });
    host.on('hide', () => { shown = false; updateMotion(); });
    host.ready().then(() => {
      ready = true;
      refreshStatus();
    }).catch(() => {
      supported = false;
      setStatus('暂时无法连接系统壁纸设置。');
    });
  }
})();
