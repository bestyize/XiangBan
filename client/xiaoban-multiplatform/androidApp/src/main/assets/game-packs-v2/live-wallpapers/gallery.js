(() => {
  const preview = document.querySelector('#preview');
  const choices = [...document.querySelectorAll('[data-scene]')];
  const button = document.querySelector('#apply');
  const status = document.querySelector('#status');
  const copy = {
    aquarium: ['水下花园', '轻触水面，泛起涟漪'],
    galaxy: ['银河', '轻触星空，拨动视角'],
  };
  const defaultStatus = status.textContent;
  let selected;
  let scene;
  let shown = true;
  let supported = false;

  function updateVisibility() {
    scene?.setActive(shown && !document.hidden);
  }

  function select(id) {
    if (id === selected) return;
    scene?.destroy();
    preview.querySelector('canvas')?.remove();
    const canvas = document.createElement('canvas');
    canvas.setAttribute('aria-hidden', 'true');
    preview.prepend(canvas);
    selected = id;
    scene = window.createWallpaperScene(canvas, id, 30);
    canvas.addEventListener('pointerup', event => {
      const bounds = canvas.getBoundingClientRect();
      scene.tap((event.clientX - bounds.left) / bounds.width, (event.clientY - bounds.top) / bounds.height);
    });
    const [title, note] = copy[id];
    document.querySelector('#scene-name').textContent = title;
    document.querySelector('#scene-note').textContent = note;
    preview.setAttribute('aria-label', title + '实时预览');
    for (const choice of choices) choice.setAttribute('aria-pressed', String(choice.dataset.scene === id));
    updateVisibility();
  }

  for (const choice of choices) choice.addEventListener('click', () => select(choice.dataset.scene));
  document.addEventListener('visibilitychange', updateVisibility);
  window.xb?.on('show', () => { shown = true; updateVisibility(); });
  window.xb?.on('hide', () => { shown = false; updateVisibility(); });

  button.addEventListener('click', async () => {
    if (!supported) return;
    button.disabled = true;
    status.textContent = defaultStatus;
    try {
      await window.xb.liveWallpapers.preview(selected);
    } catch (_) {
      status.textContent = '未能打开系统壁纸预览，请重试。';
    } finally {
      button.disabled = false;
    }
  });

  select('aquarium');
  if (window.xb) {
    window.xb.ready().then(async () => {
      supported = await window.xb.liveWallpapers.supported();
      button.disabled = !supported;
      if (!supported) status.textContent = '这台设备的系统设置暂不可用，仍可预览壁纸。';
    }).catch(() => { status.textContent = '暂时无法连接系统壁纸设置。'; });
  }
})();
