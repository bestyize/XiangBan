import './community-chrome.css'

type TauriWindow = Window & { __TAURI_INTERNALS__?: { invoke: (command: string, args: Record<string, unknown>) => Promise<unknown> } }
const invoke = (action: string) => (window as TauriWindow).__TAURI_INTERNALS__?.invoke('community_chrome_action', { action }).catch(() => undefined)
const root = document.getElementById('app')!
root.innerHTML = `<header class="titlebar"><div class="controls"><button type="button" data-action="close" aria-label="关闭">×</button><button type="button" data-action="minimize" aria-label="最小化">−</button><button type="button" data-action="maximize" aria-label="最大化或还原">□</button></div><span>小伴社区</span><button type="button" class="fullscreen" data-action="fullscreen" aria-label="切换全屏">⛶</button></header>${['n','s','e','w','ne','nw','se','sw'].map(edge => `<div class="edge ${edge}" data-edge="${edge}"></div>`).join('')}`
root.addEventListener('click', event => {
  const button = (event.target as HTMLElement).closest<HTMLButtonElement>('[data-action]')
  if (button?.dataset.action) void invoke(button.dataset.action)
})
root.addEventListener('mousedown', event => {
  if (event.button !== 0) return
  const target = event.target as HTMLElement
  const edge = target.closest<HTMLElement>('[data-edge]')?.dataset.edge
  if (edge) { event.preventDefault(); void invoke(edge) }
  else if (target.closest('.titlebar') && !target.closest('button')) { event.preventDefault(); void invoke(event.detail === 2 ? 'maximize' : 'drag') }
})
document.addEventListener('keydown', event => {
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'w') { event.preventDefault(); void invoke('close') }
})
