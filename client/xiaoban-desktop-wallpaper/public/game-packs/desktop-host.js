/* Desktop-only adapter: escape always returns, web-only account/share UI is hidden. */
(() => {
  const nonce = new URLSearchParams(location.search).get('hostNonce')
  if (!nonce || window.parent === window) return
  const style = document.createElement('style')
  style.textContent = '#shareResultButton,.result-share-button,.candy-wallet,.result-wallet,#fullscreenButton{display:none!important}'
  document.head.append(style)
  document.addEventListener('DOMContentLoaded', () => {
    const button = document.getElementById('shareResultButton')
    if (button) button.hidden = true
  })
  window.addEventListener('keydown', event => {
    if (event.key !== 'Escape') return
    event.preventDefault(); event.stopImmediatePropagation()
    window.parent.postMessage({channel:'xiaban.game-pack',protocol:1,type:'game.exit',nonce}, '*')
  }, true)
  let tested = false
  window.addEventListener('message', async event => {
    const d = event.data
    if (event.source !== window.parent || !d || d.channel !== 'xiaban.game-host' || d.protocol !== 1 || d.nonce !== nonce) return
    if (d.type === 'host.init') document.body.classList.add('desktop-hosted')
    if (d.type === 'host.smoke.escape' && new URLSearchParams(location.search).get('desktopSmoke') === '1') {
      window.dispatchEvent(new KeyboardEvent('keydown', { key:'Escape', bubbles:true, cancelable:true }))
    }
    if (d.type === 'host.init' && !tested && new URLSearchParams(location.search).get('desktopSmoke') === '1') {
      tested = true
      let nativeAccess = false, parentAccess = false
      try { parentAccess = !!window.parent.__XIAOBAN_GAME_TOKEN__ } catch {}
      try {
        const invoke = window.__TAURI__?.core?.invoke || window.__TAURI_INTERNALS__?.invoke
        if (invoke) { await invoke('get_settings'); nativeAccess = true }
      } catch {}
      window.parent.postMessage({channel:'xiaban.game-pack',protocol:1,type:'game.security',nonce,nativeAccess,parentAccess}, '*')
    }
  })
})()
