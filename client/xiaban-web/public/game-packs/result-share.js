/* Shared end-of-round sharing. Packs remain independent of React and account data. */
(() => {
  let nonce = new URLSearchParams(location.search).get('hostNonce') || ''
  let hosted = false
  window.addEventListener('message', (event) => {
    const data = event.data
    if (event.source !== window.parent || !data || data.channel !== 'xiaban.game-host' || data.protocol !== 1 || data.type !== 'host.init' || data.nonce !== nonce) return
    hosted = true
  })
  const button = document.getElementById('shareResultButton')
  let result = null
  window.XiabanResultShare = {
    finish(value) { result = { lines: 0, maxTile: 0, level: 0, steps: 0, ...value }; if (button) button.hidden = false },
    reset() { result = null; if (button) button.hidden = true },
  }
  button?.addEventListener('click', () => {
    if (!result) return
    if (hosted && window.parent !== window) {
      window.parent.postMessage({ channel: 'xiaban.game-pack', protocol: 1, type: 'game.share', nonce, result }, '*')
    } else if (window.parent === window) {
      const hash = new URLSearchParams({ v: '1', s: String(result.score), l: String(result.lines), m: String(result.maxTile), n: String(result.level), p: String(result.steps) })
      location.href = `/games/${result.gameId}/share?own=1#${hash}`
    }
  })
})()
