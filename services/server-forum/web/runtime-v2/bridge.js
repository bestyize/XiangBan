/* Loaded before work scripts from the work's own origin. */
(() => {
  'use strict'
  const script = document.currentScript
  const parentOrigin = script.dataset.parentOrigin
  const channel = 'xiaoban.app-runtime.v2'
  const events = new Map()
  const zeroInsets = Object.freeze({ top: 0, right: 0, bottom: 0, left: 0 })
  const capsule = Object.freeze({ top: 0, right: 0, width: 0, height: 0 })
  const pending = new Map()
  let sequence = 0
  let scoreStarted = false
  let ready = false
  let visible = !document.hidden

  function emit(name) {
    for (const listener of events.get(name) || []) {
      try { listener() } catch (error) { console.error(error) }
    }
  }

  function visibility(next) {
    if (next === visible) return
    visible = next
    emit(visible ? 'show' : 'hide')
  }

  function send(type) {
    if (parent !== window && parentOrigin) parent.postMessage({ channel, type }, parentOrigin)
  }

  document.addEventListener('visibilitychange', () => visibility(!document.hidden))
  window.addEventListener('pagehide', () => visibility(false))
  window.addEventListener('pageshow', () => visibility(!document.hidden))
  window.addEventListener('message', event => {
    if (event.source !== parent || event.origin !== parentOrigin || event.data?.channel !== channel) return
    if (event.data.type === 'show') visibility(!document.hidden)
    if (event.data.type === 'hide') visibility(false)
    if (event.data.type === 'storage-result') {
      const request = pending.get(event.data.id)
      if (!request) return
      pending.delete(event.data.id)
      clearTimeout(request.timer)
      if (event.data.error) request.reject(failure(event.data.error, '作品存储操作失败'))
      else request.resolve(event.data.value)
    }
  })

  const orientation = matchMedia('(orientation: landscape)')
  const dark = matchMedia('(prefers-color-scheme: dark)')
  orientation.addEventListener('change', () => emit('orientation'))
  dark.addEventListener('change', () => emit('theme'))
  for (const key of Object.keys(zeroInsets)) document.documentElement.style.setProperty('--xb-safe-' + key, '0px')
  for (const key of Object.keys(capsule)) document.documentElement.style.setProperty('--xb-capsule-' + key, '0px')

  function failure(code, message) { return Object.assign(new Error(message), { code }) }
  async function storageRequest(operation, key, value) {
    if (parent === window || !parentOrigin) throw failure('unsupported', '当前页面没有小伴宿主')
    const id = ++sequence
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        pending.delete(id)
        reject(failure('busy', '作品存储操作超时'))
      }, 10000)
      pending.set(id, { resolve, reject, timer })
      try { parent.postMessage({ channel, type: 'storage', id, operation, key, value }, parentOrigin) }
      catch {
        pending.delete(id)
        clearTimeout(timer)
        reject(failure('invalid', '存储值须为 JSON'))
      }
    })
  }

  const storage = Object.freeze({
    get: key => storageRequest('get', key),
    set: (key, value) => storageRequest('set', key, value),
    remove: key => storageRequest('remove', key),
    keys: () => storageRequest('keys'),
  })

  function checkScore(score) {
    if (!scoreStarted || !Number.isSafeInteger(score) || score < 0) throw failure('invalid', '回合或分数无效')
  }
  const game = Object.freeze({
    async start() { scoreStarted = true },
    async progress({ score }) { checkScore(score) },
    async result({ score }) { checkScore(score); scoreStarted = false },
  })
  const info = Object.freeze({
    platform: 'web', sdkVersion: 2, locale: navigator.language, safeArea: zeroInsets, capsule,
    legacySave: null, query: Object.freeze({}),
    get orientation() { return orientation.matches ? 'landscape' : 'portrait' },
    get theme() { return dark.matches ? 'dark' : 'light' },
  })

  Object.defineProperty(window, 'xb', { value: Object.freeze({
    info, storage, game,
    async ready() { if (!ready) { ready = true; send('ready') } },
    async exit() { send('exit') },
    on(name, listener) {
      if (typeof listener !== 'function') return
      if (!events.has(name)) events.set(name, new Set())
      events.get(name).add(listener)
    },
    off(name, listener) { events.get(name)?.delete(listener) },
    can() { return false },
  }) })
})()
