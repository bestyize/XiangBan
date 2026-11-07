(() => {
  const pending = new Map(), listeners = new Map(); let next = 0;
  const safeArea = __SAFE_AREA__;
  function applySafe() {
    if (!document.documentElement) return false;
    for (const [k, v] of Object.entries(xb.info.safeArea)) document.documentElement.style.setProperty(`--xb-safe-${k}`, `${v}px`);
    return true;
  }
  window.__xbSetSafeArea = area => { xb.info.safeArea = area; applySafe(); window.__xbEvent('safearea', area); };
  window.__xbEvent = (name, data) => (listeners.get(name) || []).forEach(fn => fn(data));
  function call(method, params = {}) {
    const id = ++next;
    return new Promise((resolve, reject) => { pending.set(id, {resolve, reject}); xbNative.postMessage(JSON.stringify({channel:'xiaban.app', protocol:2, kind:'call', id, method, params})); });
  }
  xbNative.onmessage = event => {
    const msg = JSON.parse(event.data);
    if (msg.channel !== 'xiaban.host' || msg.protocol !== 2 || msg.kind !== 'result') return;
    const entry = pending.get(msg.id); if (!entry) return; pending.delete(msg.id);
    msg.ok ? entry.resolve(msg.value) : entry.reject(Object.assign(new Error(msg.error?.message), msg.error));
  };
  window.xb = {info: {platform:'android', safeArea, sdkVersion:2}, ready:() => call('ready'),
    storage: {get:key => call('storage.get', {key}), set:(key,value) => call('storage.set', {key,value})},
    on:(name, fn) => { if (!listeners.has(name)) listeners.set(name, []); listeners.get(name).push(fn); }};
  if (!applySafe()) { const observer = new MutationObserver(() => { if (applySafe()) observer.disconnect(); }); observer.observe(document, {childList:true, subtree:true}); }
})();
