(() => {
  const rain = window.RainWindow;
  const options = new URLSearchParams(location.search);
  const fps = options.get('fps') === '15' ? 15 : 30;
  const width = Number(options.get('width'));
  const height = Number(options.get('height'));
  const maxPixels = fps === 15 ? 900_000 : 1_600_000;
  const fixed = Number.isInteger(width) && Number.isInteger(height) && width > 0 && height > 0 && width * height <= maxPixels;
  const sim = rain.createSim(options.get('preset'), 19, true);
  const stage = document.getElementById('wallpaper');
  let enabled = options.get('paused') !== '1';
  let destroyed = false;

  const player = rain.createPlayer((renderer, dt, still) => {
    const { clientWidth, clientHeight } = stage;
    const scale = Math.min(devicePixelRatio || 1, 2, Math.sqrt(maxPixels / Math.max(1, clientWidth * clientHeight)));
    const w = fixed ? width : Math.max(1, Math.round(clientWidth * scale));
    const h = fixed ? height : Math.max(1, Math.round(clientHeight * scale));
    const frame = renderer.draw(sim, w, h, dt, still);
    if (stage.firstChild !== frame) stage.replaceChildren(frame);
  }, fps);

  function updateMotion() {
    player.setActive(enabled && !document.hidden);
  }

  const api = {
    setActive(active) {
      enabled = Boolean(active);
      updateMotion();
    },
    setPreset(id) {
      rain.setPreset(sim, id);
      player.invalidate();
    },
    tap(x, y) {
      rain.tap(sim, x, y);
      player.invalidate();
    },
    wipe(x, y, prevX, prevY) {
      rain.wipe(sim, x, y, prevX, prevY);
      player.invalidate();
    },
    resetMist() {
      rain.resetMist(sim);
      player.invalidate();
    },
    destroy() {
      if (destroyed) return;
      destroyed = true;
      enabled = false;
      unbind();
      resize.disconnect();
      document.removeEventListener('visibilitychange', updateMotion);
      window.removeEventListener('pageshow', updateMotion);
      player.destroy();
      stage.replaceChildren();
      delete window.xbWallpaper;
    },
  };
  const unbind = rain.bindPointer(stage, {
    enabled: () => enabled && !document.hidden,
    tap: api.tap,
    wipe: api.wipe,
  });
  const resize = new ResizeObserver(player.invalidate);
  resize.observe(stage);
  document.addEventListener('visibilitychange', updateMotion);
  window.addEventListener('pageshow', updateMotion);
  window.addEventListener('pagehide', event => {
    if (event.persisted) player.setActive(false);
    else api.destroy();
  }, { passive: true });

  window.xbWallpaper = api;
  updateMotion();
})();
