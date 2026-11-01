'use strict'
// This host must run on a dedicated, cookie-free origin. It never fetches a game.
const params = new URLSearchParams(location.search)
const parentOrigin = params.get('parentOrigin'), expectedNonce = params.get('nonce')
const status = document.getElementById('status'), game = document.getElementById('game')
let initialized = false
function resolve(name) {
  if (!name || /[?#\\:]/.test(name)) return null
  const parts = []
  for (const part of name.split('/')) { if (part === '.' || !part) continue; if (part === '..') { if (!parts.length) return null; parts.pop() } else parts.push(part) }
  return parts.join('/')
}
addEventListener('message', event => {
  if (initialized || event.source !== parent || event.origin !== parentOrigin || !event.data || event.data.type !== 'xiaoban-game-init' || typeof event.data.nonce !== 'string' || event.data.nonce !== expectedNonce) return
  const { nonce, files } = event.data
  if (!files || typeof files['index.html'] !== 'string') return
  initialized = true
  try {
    const doc = new DOMParser().parseFromString(files['index.html'], 'text/html')
    // Inline local resources before handing HTML to a unique-origin, script-only frame.
    for (const script of doc.querySelectorAll('script[src]')) {
      const path = resolve(script.getAttribute('src'))
      if (!path || typeof files[path] !== 'string' || script.type === 'module') throw new Error('游戏使用了不支持的脚本引用')
      script.removeAttribute('src'); script.textContent = files[path]
    }
    for (const link of doc.querySelectorAll('link[rel="stylesheet"]')) {
      const path = resolve(link.getAttribute('href'))
      if (!path || typeof files[path] !== 'string') throw new Error('游戏使用了不支持的样式引用')
      const style = doc.createElement('style'); style.textContent = files[path]; link.replaceWith(style)
    }
    const csp = doc.createElement('meta'); csp.httpEquiv = 'Content-Security-Policy'; csp.content = "default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; img-src data:; font-src data:; media-src data:; connect-src 'none'; object-src 'none'; frame-src 'none'; base-uri 'none'; form-action 'none'; navigate-to 'none'"
    doc.head.prepend(csp)
    const frame = document.createElement('iframe'); frame.setAttribute('sandbox','allow-scripts'); frame.setAttribute('credentialless',''); frame.referrerPolicy = 'no-referrer'; frame.title = '游戏试玩'; frame.srcdoc = '<!doctype html>' + doc.documentElement.outerHTML
    frame.addEventListener('load', () => { status.hidden = true; parent.postMessage({ type:'xiaoban-game-ready', nonce }, parentOrigin) }, { once:true })
    game.replaceChildren(frame)
  } catch (error) { status.textContent = error instanceof Error ? error.message : '游戏加载失败'; parent.postMessage({ type:'xiaoban-game-error', nonce, message:status.textContent }, parentOrigin) }
})
if (parentOrigin) parent.postMessage({type:'xiaoban-game-host-ready', nonce:expectedNonce}, parentOrigin)
