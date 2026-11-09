(() => {
  if (window !== window.top) return;

  const initialInfo = __XB_INITIAL_INFO__;
  const capabilities = new Set(__XB_CAPABILITIES__);
  const pending = new Map();
  const listeners = new Map();
  let nextId = 0;
  let ready = false;

  function applyCss() {
    const root = document.documentElement;
    if (!root) return false;
    for (const [prefix, values, keys] of [
      ['safe', xb.info.safeArea, ['top', 'right', 'bottom', 'left']],
      ['capsule', xb.info.capsule, ['top', 'right', 'width', 'height']],
    ]) {
      for (const key of keys) {
        const value = values && values[key];
        root.style.setProperty(`--xb-${prefix}-${key}`, `${Number.isFinite(value) ? value : 0}px`);
      }
    }
    return true;
  }

  function call(method, params = {}) {
    const id = ++nextId;
    return new Promise((resolve, reject) => {
      pending.set(id, { resolve, reject });
      try {
        xbNative.postMessage(JSON.stringify({ channel: 'xiaban.app', protocol: 2, kind: 'call', id, method, params }));
      } catch (error) {
        pending.delete(id);
        reject(error);
      }
    });
  }

  function emit(name, data) {
    for (const listener of [...(listeners.get(name) || [])]) {
      try { listener(data); } catch (error) { console.error(error); }
    }
  }

  const xb = window.xb = {
    info: initialInfo,
    ready() { ready = true; return call('ready'); },
    on(name, fn) {
      if (typeof fn !== 'function') return;
      if (!listeners.has(name)) listeners.set(name, new Set());
      listeners.get(name).add(fn);
    },
    off(name, fn) { listeners.get(name)?.delete(fn); },
    can(name) { return capabilities.has(name); },
    exit() { return call('exit'); },
    exportMbti(payload) { return call('builtin.mbti.export', payload); },
    remote: {
      supported: () => call('builtin.remote.supported'),
      catalog: () => call('builtin.remote.catalog'),
      send: (profile, key, state) => call('builtin.remote.send', { profile, key, state }),
    },
    storage: {
      get: key => call('storage.get', { key }),
      set: (key, value) => call('storage.set', { key, value }),
      remove: key => call('storage.remove', { key }),
      keys: () => call('storage.keys'),
    },
    game: {
      start: () => call('game.start'),
      progress: value => call('game.progress', value),
      result: value => call('game.result', value),
    },
  };

  xbNative.onmessage = event => {
    let message;
    try { message = JSON.parse(event.data); } catch (_) { return; }
    if (!message || message.channel !== 'xiaban.host' || message.protocol !== 2) return;
    if (message.kind === 'result') {
      const request = pending.get(message.id);
      if (!request) return;
      pending.delete(message.id);
      if (message.ok === true) request.resolve(message.value);
      else if (message.ok === false) request.reject(Object.assign(new Error(message.error?.message || 'Host request failed'), { code: message.error?.code }));
      else pending.set(message.id, request);
      return;
    }
    if (message.kind !== 'event' || !['show', 'hide', 'safearea', 'orientation', 'theme'].includes(message.name)) return;
    const data = message.data;
    if (message.name === 'safearea' && data && typeof data === 'object') {
      if (data.safeArea) xb.info.safeArea = data.safeArea;
      if (data.capsule) xb.info.capsule = data.capsule;
      applyCss();
    } else if (message.name === 'orientation' && typeof data === 'string') xb.info.orientation = data;
    else if (message.name === 'theme' && typeof data === 'string') xb.info.theme = data;
    emit(message.name, data);
  };

  if (!applyCss()) {
    const observer = new MutationObserver(() => { if (applyCss()) observer.disconnect(); });
    observer.observe(document, { childList: true, subtree: true });
  }

  addEventListener('error', event => {
    if (ready) return;
    let filename = '';
    try { filename = new URL(event.filename || '', location.href).pathname.split('/').pop().slice(0, 128); } catch (_) {}
    call('document.error', {
      filename,
      type: String(event.error?.name || 'Error').slice(0, 64),
      message: String(event.message || '').slice(0, 256),
      line: Number.isFinite(event.lineno) ? event.lineno : 0,
    }).catch(() => {});
  });

  call('document.connect').then(value => {
    if (value && value.info && typeof value.info === 'object') {
      Object.assign(xb.info, value.info);
      applyCss();
    }
  }).catch(() => {});
})();
