(() => {
  const host = window.xb;
  const wallpapers = host?.liveWallpapers;
  const rain = window.RainWindow;
  const library = document.getElementById('library');
  const preview = document.getElementById('preview');
  const back = document.getElementById('back');
  const apply = document.getElementById('apply');
  const status = document.getElementById('status');
  const toast = document.getElementById('toast');
  const hint = document.getElementById('gesture-hint');
  const dock = preview.querySelector('.preview-dock');
  const presetButtons = [];
  const modeButtons = preview.querySelectorAll('button[data-mode]');
  const overlays = preview.querySelectorAll('[data-overlay]');
  const darkTheme = matchMedia('(prefers-color-scheme: dark)');
  const timeFormat = new Intl.DateTimeFormat('zh-CN', { hour: '2-digit', minute: '2-digit', hourCycle: 'h23' });
  const weekdayFormat = new Intl.DateTimeFormat('zh-CN', { weekday: 'short' });

  let preset = rain.presets[0];
  let mode = 'wallpaper';
  let previewOpen = false;
  let visit = 0;
  let opener;
  let shown = true;
  let ready = false;
  let supported = false;
  let applying = false;
  let statusRequest = 0;
  let switchTargets = {};
  let availability = wallpapers ? '正在检查系统壁纸设置…' : '在小伴 Android App 内可设为壁纸。';
  let feedback = '';
  let toastTimer;

  const galleryScene = {
    canvas: library.querySelector('.hero-canvas'),
    sim: rain.createSim(preset.id, 7, true),
  };
  const previewScene = {
    canvas: preview.querySelector('.preview-canvas'),
    sim: rain.createSim(preset.id, 8, true),
  };

  const player = rain.createPlayer((renderer, dt, still) => {
    const { canvas, sim } = previewOpen ? previewScene : galleryScene;
    const { width, height } = canvas.getBoundingClientRect();
    if (width < 1 || height < 1) return;
    const scale = Math.min(1, 390 / width, 812 / height);
    const w = Math.max(1, Math.round(width * scale));
    const h = Math.max(1, Math.round(height * scale));
    if (canvas.width !== w) canvas.width = w;
    if (canvas.height !== h) canvas.height = h;
    const frame = renderer.draw(sim, w, h, dt, still);
    canvas.getContext('2d', { alpha: false })?.drawImage(frame, 0, 0);
  }, navigator.deviceMemory && navigator.deviceMemory <= 2 ? 15 : 30);

  const resize = new ResizeObserver(entries => {
    for (const entry of entries) {
      if (entry.target === dock) preview.style.setProperty('--dock-height', dock.offsetHeight + 'px');
    }
    player.invalidate();
  });
  for (const element of [galleryScene.canvas, previewScene.canvas, dock]) resize.observe(element);

  function setTheme(theme = host?.info?.theme) {
    document.documentElement.dataset.theme = theme || (darkTheme.matches ? 'dark' : 'light');
  }

  function updateControls() {
    const label = {
      home: '切换桌面壁纸',
      lock: '切换锁屏壁纸',
      both: '切换桌面和锁屏壁纸',
    }[switchTargets[mode]];
    apply.textContent = applying ? '正在应用…' : label || '设为壁纸';
    apply.disabled = applying || !supported;
    status.textContent = feedback || availability;
    status.hidden = !status.textContent;
  }

  function notify(message) {
    clearTimeout(toastTimer);
    toast.textContent = message;
    toast.hidden = false;
    toastTimer = setTimeout(() => { toast.hidden = true; }, 2400);
  }

  function updateSelection(current) {
    switchTargets = current.switchTargets || {};
    for (const row of library.querySelectorAll('[data-slot]')) {
      const selected = current[row.dataset.slot];
      row.hidden = !selected;
      row.querySelector('em').textContent = selected ? '(' + rain.getPreset(selected).name + ')' : '';
    }

    let label = '';
    if (current.home && current.lock) {
      label = switchTargets.wallpaper === 'both' ? '桌面和锁屏使用中' : '桌面与锁屏各自使用中';
    } else if (current.home) label = '桌面使用中';
    else if (current.lock) label = '锁屏使用中';

    document.getElementById('slot-strip').hidden = !label;
    const badge = document.getElementById('applied-badge');
    badge.hidden = !label;
    badge.querySelector('span').textContent = label;
  }

  async function refreshStatus() {
    if (!ready || applying) return;
    const request = ++statusRequest;
    try {
      const current = await wallpapers.status();
      if (request !== statusRequest) return;
      supported = current.supported;
      availability = supported ? '' : '此设备当前无法应用动态壁纸。';
      updateSelection(current);
      if (current.fps) player.setFps(current.fps);
    } catch (_) {
      if (request !== statusRequest) return;
      supported = false;
      availability = '暂时无法读取系统壁纸状态，请重新打开再试。';
    }
    updateControls();
  }

  function setPreset(id) {
    preset = rain.getPreset(id);
    rain.setPreset(galleryScene.sim, preset.id);
    rain.setPreset(previewScene.sim, preset.id);
    for (const button of presetButtons) button.setAttribute('aria-pressed', String(button.dataset.preset === preset.id));
    document.getElementById('hero-preset').textContent = '可拭雾 · ' + preset.name;
    document.getElementById('preview-preset').textContent = preset.name + ' · ' + preset.desc;
    player.invalidate();
  }

  for (const rail of document.querySelectorAll('.preset-rail, .dock-presets')) {
    for (const item of rain.presets) {
      const button = document.createElement('button');
      button.type = 'button';
      button.className = rail.classList.contains('preset-rail') ? 'preset-chip' : 'dock-preset';
      button.dataset.preset = item.id;
      button.setAttribute('aria-pressed', String(item === preset));
      const orb = document.createElement('span');
      orb.className = 'orb';
      orb.style.background = item.swatch;
      orb.setAttribute('aria-hidden', 'true');
      const name = document.createElement('span');
      name.textContent = item.name;
      button.append(orb, name);
      button.addEventListener('click', () => setPreset(item.id));
      rail.append(button);
      presetButtons.push(button);
    }
  }

  function updateMotion() {
    player.setActive(shown && !document.hidden);
  }

  function setMode(next) {
    mode = next;
    preview.dataset.mode = mode;
    for (const button of modeButtons) button.setAttribute('aria-pressed', String(button.dataset.mode === mode));
    for (const overlay of overlays) {
      overlay.hidden = overlay.dataset.overlay !== mode;
      if (overlay.hidden) continue;
      const now = new Date();
      overlay.querySelector('[data-time]').textContent = timeFormat.format(now);
      overlay.querySelector('[data-date]').textContent = `${now.getMonth() + 1}月${now.getDate()}日 ${weekdayFormat.format(now)}`;
    }
    updateControls();
  }

  function setPage(open) {
    if (open === previewOpen) return;
    previewOpen = open;
    visit++;
    if (open) {
      feedback = '';
      toast.hidden = true;
      setMode(mode);
    }
    library.classList.toggle('pushed', open);
    library.inert = open;
    preview.classList.toggle('is-hidden', !open);
    preview.inert = !open;
    wallpapers?.previewing(open).catch(() => {});
    player.invalidate();

    const focus = open ? back : opener && !opener.closest('[hidden]') ? opener : library.querySelector('.hero-enter');
    focus.focus({ preventScroll: true });
  }

  for (const button of library.querySelectorAll('[data-open-preview]')) {
    button.addEventListener('click', () => {
      if (previewOpen) return;
      opener = button;
      history.pushState({ rainWindowPreview: true }, '');
      setPage(true);
    });
  }
  for (const button of modeButtons) button.addEventListener('click', () => setMode(button.dataset.mode));
  back.addEventListener('click', () => history.back());
  window.addEventListener('popstate', event => setPage(event.state?.rainWindowPreview === true));

  const unbind = [galleryScene, previewScene].map(({ canvas, sim }) => {
    const isPreview = sim === previewScene.sim;
    function changed() {
      if (isPreview) hint.classList.add('faded');
      player.invalidate();
    }
    return rain.bindPointer(canvas, {
      enabled: () => shown && !document.hidden && isPreview === previewOpen && (!isPreview || mode !== 'lock'),
      tap(x, y) { rain.tap(sim, x, y); changed(); },
      wipe(x, y, prevX, prevY) { rain.wipe(sim, x, y, prevX, prevY); changed(); },
    });
  });

  document.getElementById('reset-mist').addEventListener('click', () => {
    rain.resetMist(previewScene.sim);
    player.invalidate();
  });

  apply.addEventListener('click', async () => {
    if (applying || !supported || !previewOpen) return;
    const requested = preset;
    const requestedVisit = visit;
    applying = true;
    statusRequest++;
    feedback = '';
    updateControls();
    try {
      const result = await wallpapers.preview(requested.id, mode);
      updateSelection(result);
      if (visit !== requestedVisit) return;
      if (result.outcome === 'applied') {
        const switched = {
          home: '已切换桌面壁纸为',
          lock: '已切换锁屏壁纸为',
          both: '已原地切换桌面与锁屏为',
        }[result.switchedTarget];
        history.back();
        notify(switched ? switched + '「' + requested.name + '」' : '壁纸已应用');
      } else if (result.outcome === 'unverified') {
        feedback = '系统已确认，暂时无法核实壁纸的使用位置。';
      } else {
        feedback = '没有检测到新的小伴壁纸。请在系统壁纸预览中确认应用。';
      }
    } catch (error) {
      if (visit !== requestedVisit) return;
      if (error.code === 'gesture') feedback = '请再点一次按钮。';
      else if (error.code === 'save_failed') feedback = '未能保存壁纸，请重试。';
      else feedback = '未能打开系统动态壁纸预览，请重试。';
    } finally {
      applying = false;
      updateControls();
    }
  });

  document.addEventListener('visibilitychange', updateMotion);
  window.addEventListener('pageshow', updateMotion);
  window.addEventListener('pagehide', event => {
    player.setActive(false);
    if (event.persisted) return;
    resize.disconnect();
    unbind.forEach(dispose => dispose());
    player.destroy();
  });
  darkTheme.addEventListener('change', () => setTheme());
  setTheme();
  setPage(history.state?.rainWindowPreview === true);
  updateControls();
  updateMotion();

  if (wallpapers) {
    host.on('theme', setTheme);
    host.on('show', () => {
      shown = true;
      updateMotion();
      if (previewOpen) setMode(mode);
      refreshStatus();
    });
    host.on('hide', () => { shown = false; updateMotion(); });
    requestAnimationFrame(() => {
      host.ready().then(() => {
        ready = true;
        refreshStatus();
      }).catch(() => {
        supported = false;
        availability = '暂时无法连接系统壁纸设置。';
        updateControls();
      });
    });
  }
})();
