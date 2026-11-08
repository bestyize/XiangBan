(() => {
  'use strict'
  const xb = globalThis.xb
  if (!xb) return

  const saveKey = 'save'
  const root = document.documentElement
  root.classList.add('xb-builtin-runtime')
  const reportError = error => console.error('小伴内置作品：', error)

  function layout() {
    const safeArea = { ...xb.info.safeArea }
    const capsule = xb.info.capsule
    if (xb.info.orientation === 'portrait') {
      safeArea.top = Math.max(safeArea.top, capsule.top + capsule.height + 8)
    }
    return { safeArea, capsule }
  }

  async function connect(initialize, options = {}) {
    if (options.hide) xb.on('hide', options.hide)
    if (options.show) xb.on('show', options.show)
    if (options.layout) xb.on('safearea', () => options.layout(layout()))
    let save = null
    if (options.save) {
      try {
        save = await xb.storage.get(saveKey)
        if (save === null && xb.info.legacySave !== null) {
          save = xb.info.legacySave
          await xb.storage.set(saveKey, save)
        }
      } catch (error) {
        reportError(error)
      }
    } else {
      // The shared gameplay script finishes synchronous initialization first.
      await Promise.resolve()
    }
    document.body.classList.add('desktop-hosted')
    initialize({ ...layout(), save })
  }

  function send(type, payload = {}) {
    let result
    switch (type) {
      case 'game.ready': result = xb.ready(); break
      case 'game.start': result = xb.game.start(); break
      case 'game.progress': result = xb.game.progress({ score: payload.score }); break
      case 'game.result': result = xb.game.result({ score: payload.score }); break
      case 'game.save': result = xb.storage.set(saveKey, payload.data); break
      case 'game.exit': result = xb.exit(); break
    }
    result?.catch(reportError)
  }

  globalThis.XiaobanBuiltinRuntime = Object.freeze({
    connect(initialize, options) { connect(initialize, options).catch(reportError) },
    send,
  })
})()
