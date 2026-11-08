/* Protocol 1: register before game initialization. */
window.GameHost = (() => {
  const nonce = new URLSearchParams(location.search).get('hostNonce');
  let ready = false, pending = null, onload = null, saved = null;
  function send(type, extra = {}) { if (ready) parent.postMessage({channel:'xiaban.game-pack',protocol:1,nonce,type,...extra}, '*'); }
  addEventListener('message', e => {
    const m = e.data;
    if (e.source !== parent || !m || m.channel !== 'xiaban.game-host' || m.protocol !== 1 || m.nonce !== nonce || m.type !== 'host.init') return;
    if (ready) { send('game.ready'); return; }
    ready = true; saved = m.save; send('game.ready');
    if (onload) onload(saved);
    if (pending) { send('game.save', {data:pending}); pending = null; }
  });
  return {send, save(data) { if (ready) send('game.save',{data}); else pending=data; }, restore(fn) { onload=fn; if (ready) fn(saved); }};
})();
