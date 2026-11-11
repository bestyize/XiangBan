(() => {
  'use strict';
  const key = 'pour-joy-v2', sdk = globalThis.xb;
  let ready = false, queue = Promise.resolve(), lastError = null;
  function signal(error) { lastError = error; globalThis.dispatchEvent(new CustomEvent('joy-storage-error')); }
  const host = {
    get native() { return !!sdk; },
    get error() { return lastError; },
    async read() {
      try { return sdk?.storage ? await sdk.storage.get(key) : JSON.parse(localStorage.getItem(key) || 'null'); }
      catch (e) { signal(e); throw e; }
    },
    save(data) {
      const snapshot = JSON.parse(JSON.stringify(data));
      queue = queue.catch(() => {}).then(async () => {
        try { if (sdk?.storage) await sdk.storage.set(key, snapshot); else localStorage.setItem(key, JSON.stringify(snapshot)); lastError = null; globalThis.dispatchEvent(new CustomEvent('joy-storage-restored')); }
        catch (e) { signal(e); }
      });
      return queue;
    },
    ready() { if (ready) return; ready = true; try { Promise.resolve(sdk?.ready?.()).catch(() => {}); } catch (_) {} },
    game(type, score = 0) { try { Promise.resolve(sdk?.game?.[type]?.(type === 'start' ? undefined : { score })).catch(() => {}); } catch (_) {} },
    on(name, fn) { sdk?.on?.(name, fn); },
    query() { const id = sdk?.info?.query?.challenge || new URLSearchParams(location.search).get('challenge'); return typeof id === 'string' && /^(c\d{2}|d\d{8})$/.test(id) ? id : null; },
    haptic() { try { if (sdk?.can?.('haptics') && sdk.haptics?.impact) Promise.resolve(sdk.haptics.impact('light')).catch(() => {}); } catch (_) {} },
    canShare() { return !!(sdk?.can?.('share') && typeof sdk.share === 'function'); },
    share(payload) { return sdk.share(payload); },
    canSave() { return !!(sdk?.can?.('files') && typeof sdk.files?.save === 'function'); },
    saveImage(data, name) { return sdk.files.save({ name, mime: 'image/png', data }); },
    async copy(text) {
      if (sdk?.can?.('clipboard') && sdk.clipboard?.writeText) await sdk.clipboard.writeText(text);
      else if (navigator.clipboard?.writeText) await navigator.clipboard.writeText(text);
      else throw new Error('clipboard unavailable');
    },
    flush() { return queue; },
  };
  globalThis.PourJoyHost = Object.freeze(host);
})();
