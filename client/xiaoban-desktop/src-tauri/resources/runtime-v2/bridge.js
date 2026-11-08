/* Xiaoban Desktop App Runtime V2 Bridge SDK */
(() => {
  'use strict';
  const invoke = window.__TAURI__?.core?.invoke;
  try {
    delete window.__TAURI__;
  } catch {}

  const events = new Map();
  const zeroInsets = Object.freeze({ top: 0, right: 0, bottom: 0, left: 0 });
  const capsule = Object.freeze({ top: 0, right: 0, width: 0, height: 0 });
  let scoreStarted = false;
  let ready = false;
  let visible = !document.hidden;

  function emit(name, data) {
    for (const listener of events.get(name) || []) {
      try { listener(data); } catch (e) { console.error(e); }
    }
  }

  function visibility(next) {
    if (next === visible) return;
    visible = next;
    emit(visible ? 'show' : 'hide');
  }

  document.addEventListener('visibilitychange', () => visibility(!document.hidden));
  window.addEventListener('pagehide', () => visibility(false));
  window.addEventListener('pageshow', () => visibility(!document.hidden));

  const dark = matchMedia('(prefers-color-scheme: dark)');
  dark.addEventListener('change', () => emit('theme', dark.matches ? 'dark' : 'light'));

  for (const key of Object.keys(zeroInsets)) document.documentElement.style.setProperty('--xb-safe-' + key, '0px');
  for (const key of Object.keys(capsule)) document.documentElement.style.setProperty('--xb-capsule-' + key, '0px');

  function failure(code, message) { return Object.assign(new Error(message), { code }); }

  const storage = Object.freeze({
    async get(key) {
      if (!invoke) throw failure('unsupported', '当前环境未提供小伴存储服务');
      return invoke('app_runtime_storage_get', { key });
    },
    async set(key, value) {
      if (!invoke) throw failure('unsupported', '当前环境未提供小伴存储服务');
      return invoke('app_runtime_storage_set', { key, value });
    },
    async remove(key) {
      if (!invoke) throw failure('unsupported', '当前环境未提供小伴存储服务');
      return invoke('app_runtime_storage_remove', { key });
    },
    async keys() {
      if (!invoke) throw failure('unsupported', '当前环境未提供小伴存储服务');
      return invoke('app_runtime_storage_keys');
    }
  });

  function checkScore(score) {
    if (!scoreStarted || !Number.isSafeInteger(score) || score < 0) throw failure('invalid', '回合或分数无效');
  }

  const game = Object.freeze({
    async start() { scoreStarted = true; },
    async progress({ score }) {
      checkScore(score);
      if (invoke) await invoke('app_runtime_game_progress', { score }).catch(() => {});
    },
    async result({ score }) {
      checkScore(score);
      scoreStarted = false;
      if (invoke) await invoke('app_runtime_game_result', { score }).catch(() => {});
    }
  });

  const info = Object.freeze({
    platform: 'desktop',
    sdkVersion: 2,
    locale: navigator.language,
    safeArea: zeroInsets,
    capsule,
    legacySave: null,
    query: Object.freeze({}),
    get orientation() { return window.innerWidth >= window.innerHeight ? 'landscape' : 'portrait'; },
    get theme() { return dark.matches ? 'dark' : 'light'; }
  });

  Object.defineProperty(window, 'xb', {
    value: Object.freeze({
      info,
      storage,
      game,
      async ready() {
        if (!ready) {
          ready = true;
          if (invoke) await invoke('app_runtime_ready').catch(() => {});
        }
      },
      async exit() {
        if (invoke) await invoke('app_runtime_exit').catch(() => {});
      },
      on(name, listener) {
        if (typeof listener !== 'function') return;
        if (!events.has(name)) events.set(name, new Set());
        events.get(name).add(listener);
      },
      off(name, listener) {
        events.get(name)?.delete(listener);
      },
      can(capability) {
        if (['clipboard', 'screen', 'files'].includes(capability)) return true;
        return false;
      }
    }),
    configurable: false,
    writable: false
  });
})();
