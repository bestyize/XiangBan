window.addEventListener('DOMContentLoaded', () => {
  if (!window.atlasApp) return
  if (window.xb?.ready) {
    void window.xb.ready()
  } else if (window.parent !== window) {
    const parentOrigin = location.hostname === 'res.xiangban.xyz' ? 'https://chat.xiangban.xyz' : location.origin
    window.parent.postMessage({ channel: 'xiaoban.app-runtime.v2', type: 'ready' }, parentOrigin)
  }
}, { once: true })
