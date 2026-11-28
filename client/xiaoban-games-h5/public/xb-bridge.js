/* 小伴游戏厅 Web 协议 2 运行时桥接 (window.xb) */
(function () {
  if (window.xb) return;

  var gameId = window.__XB_GAME_ID__ || '';
  if (!gameId) {
    var m = location.pathname.match(/\/game-packs\/([a-z0-9_-]+)\//i);
    if (m) gameId = m[1];
  }
  var STORE_PREFIX = 'xb_game_store:' + gameId + ':';
  var listeners = Object.create(null);

  function parseParams() {
    var out = Object.create(null);
    try {
      var sp = new URLSearchParams(location.search);
      sp.forEach(function (v, k) {
        if (k !== 'hostNonce' && k !== 'xbTheme' && k !== 'xbMuted') out[k] = v;
      });
    } catch (_) {}
    return out;
  }

  function initialTheme() {
    try {
      var sp = new URLSearchParams(location.search);
      if (sp.get('xbTheme') === 'dark') return 'dark';
      if (sp.get('xbTheme') === 'light') return 'light';
    } catch (_) {}
    return window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }

  var info = {
    platform: 'web',
    orientation: window.innerWidth >= window.innerHeight ? 'landscape' : 'portrait',
    safeArea: { top: 0, right: 0, bottom: 0, left: 0 },
    capsule: { top: 8, right: 8, width: 44, height: 44 },
    theme: initialTheme(),
    locale: navigator.language || 'zh-CN',
    query: parseParams(),
    legacySave: null,
    sdkVersion: '2.0.0-web'
  };

  function applyCssVars() {
    var r = document.documentElement;
    if (!r) return;
    r.style.setProperty('--xb-safe-top', info.safeArea.top + 'px');
    r.style.setProperty('--xb-safe-right', info.safeArea.right + 'px');
    r.style.setProperty('--xb-safe-bottom', info.safeArea.bottom + 'px');
    r.style.setProperty('--xb-safe-left', info.safeArea.left + 'px');
    r.style.setProperty('--xb-capsule-top', info.capsule.top + 'px');
    r.style.setProperty('--xb-capsule-right', info.capsule.right + 'px');
    r.style.setProperty('--xb-capsule-width', info.capsule.width + 'px');
    r.style.setProperty('--xb-capsule-height', info.capsule.height + 'px');
  }
  applyCssVars();

  function emit(name, data) {
    var arr = listeners[name];
    if (!arr) return;
    arr.slice().forEach(function (fn) {
      try { fn(data); } catch (_) {}
    });
  }

  function postHost(kind, payload) {
    try {
      if (window.parent && window.parent !== window) {
        window.parent.postMessage({
          channel: 'xiaban.app',
          protocol: 2,
          gameId: gameId,
          kind: kind,
          payload: payload || {}
        }, '*');
      }
    } catch (_) {}
  }

  function readStorage(key) {
    try {
      var raw = localStorage.getItem(STORE_PREFIX + key);
      if (raw === null) return null;
      return JSON.parse(raw);
    } catch (_) {
      return null;
    }
  }

  function writeStorage(key, value) {
    try {
      localStorage.setItem(STORE_PREFIX + key, JSON.stringify(value));
      postHost('storage.changed', { key: key });
    } catch (_) {}
  }

  function removeStorage(key) {
    try {
      localStorage.removeItem(STORE_PREFIX + key);
      postHost('storage.changed', { key: key, removed: true });
    } catch (_) {}
  }

  function listKeys() {
    var keys = [];
    try {
      for (var i = 0; i < localStorage.length; i++) {
        var k = localStorage.key(i);
        if (k && k.indexOf(STORE_PREFIX) === 0) {
          keys.push(k.slice(STORE_PREFIX.length));
        }
      }
    } catch (_) {}
    return keys;
  }

  var wakeLock = null;
  var supportedCaps = {
    haptics: typeof navigator.vibrate === 'function',
    share: true,
    clipboard: !!(navigator.clipboard && navigator.clipboard.writeText),
    screen: !!(navigator.wakeLock && navigator.wakeLock.request),
    files: true
  };

  window.xb = {
    info: info,
    ready: function () {
      postHost('ready');
      return Promise.resolve();
    },
    exit: function () {
      postHost('exit');
      return Promise.resolve();
    },
    on: function (name, fn) {
      if (!listeners[name]) listeners[name] = [];
      if (listeners[name].indexOf(fn) === -1) listeners[name].push(fn);
    },
    off: function (name, fn) {
      if (!listeners[name]) return;
      listeners[name] = listeners[name].filter(function (f) { return f !== fn; });
    },
    can: function (cap) {
      return !!supportedCaps[cap];
    },
    storage: {
      get: function (key) { return Promise.resolve(readStorage(String(key))); },
      set: function (key, val) { writeStorage(String(key), val); return Promise.resolve(); },
      remove: function (key) { removeStorage(String(key)); return Promise.resolve(); },
      keys: function () { return Promise.resolve(listKeys()); }
    },
    game: {
      start: function () { postHost('game.start'); return Promise.resolve(); },
      progress: function (data) {
        var score = data && typeof data.score === 'number' ? Math.max(0, Math.floor(data.score)) : 0;
        postHost('game.progress', { score: score });
        return Promise.resolve();
      },
      result: function (data) {
        var score = data && typeof data.score === 'number' ? Math.max(0, Math.floor(data.score)) : 0;
        postHost('game.result', { score: score });
        return Promise.resolve();
      }
    },
    haptics: {
      impact: function (style) {
        try {
          if (navigator.vibrate) navigator.vibrate(style === 'heavy' ? 28 : style === 'medium' ? 16 : 8);
        } catch (_) {}
        return Promise.resolve();
      },
      notify: function (type) {
        try {
          if (navigator.vibrate) navigator.vibrate(type === 'error' ? [24, 40, 24] : 14);
        } catch (_) {}
        return Promise.resolve();
      }
    },
    clipboard: {
      writeText: function (text) {
        if (navigator.clipboard && navigator.clipboard.writeText) {
          return navigator.clipboard.writeText(String(text || ''));
        }
        return Promise.reject(Object.assign(new Error('unsupported'), { code: 'unsupported' }));
      }
    },
    share: function (payload) {
      postHost('share', payload || {});
      if (navigator.share && payload) {
        return navigator.share({
          title: payload.title || document.title,
          text: payload.text || '',
          url: location.origin + '/#/play/' + gameId
        }).catch(function () {});
      }
      return Promise.resolve();
    },
    screen: {
      keepOn: function (enable) {
        if (!navigator.wakeLock || !navigator.wakeLock.request) return Promise.resolve();
        if (enable) {
          return navigator.wakeLock.request('screen').then(function (wl) { wakeLock = wl; }).catch(function () {});
        }
        if (wakeLock) {
          var prev = wakeLock; wakeLock = null;
          return prev.release().catch(function () {});
        }
        return Promise.resolve();
      }
    },
    files: {
      save: function (opts) {
        try {
          if (!opts || !opts.data) return Promise.resolve();
          var blob = opts.data instanceof Blob ? opts.data : new Blob([opts.data], { type: opts.mime || 'application/octet-stream' });
          var url = URL.createObjectURL(blob);
          var a = document.createElement('a');
          a.href = url;
          a.download = opts.name || 'xiaoban-export.png';
          document.body.appendChild(a);
          a.click();
          a.remove();
          setTimeout(function () { URL.revokeObjectURL(url); }, 15000);
        } catch (_) {}
        return Promise.resolve();
      }
    }
  };

  window.addEventListener('message', function (ev) {
    var d = ev.data;
    if (!d || d.channel !== 'xiaban.host' || d.protocol !== 2) return;
    if (d.kind === 'event') {
      if (d.name === 'safearea' && d.data) {
        if (d.data.safeArea) info.safeArea = d.data.safeArea;
        if (d.data.capsule) info.capsule = d.data.capsule;
        applyCssVars();
      } else if (d.name === 'theme' && d.data && d.data.theme) {
        info.theme = d.data.theme;
      } else if (d.name === 'orientation' && d.data && d.data.orientation) {
        info.orientation = d.data.orientation;
      }
      emit(d.name, d.data);
    }
  });

  window.addEventListener('resize', function () {
    var next = window.innerWidth >= window.innerHeight ? 'landscape' : 'portrait';
    if (next !== info.orientation) {
      info.orientation = next;
      emit('orientation', { orientation: next });
    }
  });

  document.addEventListener('visibilitychange', function () {
    emit(document.hidden ? 'hide' : 'show');
  });
})();
