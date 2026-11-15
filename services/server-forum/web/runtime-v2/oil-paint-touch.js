// Adapts the pinned Oil Paint build's existing movement and camera controls.
(() => {
  if (!matchMedia('(pointer: coarse)').matches && navigator.maxTouchPoints === 0) return
  const url = new URL(location.href)
  if (!url.searchParams.has('quality')) {
    url.searchParams.set('quality', 'low')
    try { history.replaceState(history.state, '', url) } catch { /* Opaque web frames may forbid URL changes. */ }
  }

  const keyEvent = (type, code) => window.dispatchEvent(new KeyboardEvent(type, { code, cancelable: true }))
  const releaseKeys = []
  function bindButton(button, code) {
    let held = null
    const release = () => {
      if (held === null) return
      held = null
      keyEvent('keyup', code)
    }
    releaseKeys.push(release)
    button.addEventListener('pointerdown', event => {
      if (held !== null) return
      event.preventDefault()
      held = event.pointerId
      button.setPointerCapture(held)
      keyEvent('keydown', code)
    })
    for (const event of ['pointerup', 'pointercancel', 'lostpointercapture']) button.addEventListener(event, release)
  }

  let look = null
  function stopLook() {
    if (!look) return
    look = null
    window.dispatchEvent(new MouseEvent('mouseup', { button: 2 }))
  }
  function releaseAll() {
    stopLook()
    for (const release of releaseKeys) release()
  }

  document.addEventListener('DOMContentLoaded', () => {
    document.documentElement.classList.add('xb-oil-touch')
    const controls = document.createElement('div')
    controls.className = 'xb-oil-controls'
    for (const [code, label, name] of [
      ['KeyW', '↑', '向前'], ['KeyA', '←', '向左'], ['KeyS', '↓', '向后'],
      ['KeyD', '→', '向右'], ['Space', '跳', '跳跃'],
    ]) {
      const button = document.createElement('button')
      button.type = 'button'
      button.dataset.key = code
      button.textContent = label
      button.setAttribute('aria-label', name)
      bindButton(button, code)
      controls.append(button)
    }
    document.body.append(controls)
    const view = document.getElementById('view')
    view.addEventListener('pointerdown', event => {
      if (event.pointerType !== 'touch' || look || !(event.target instanceof HTMLCanvasElement)) return
      event.preventDefault()
      look = { id: event.pointerId, x: event.clientX, y: event.clientY }
      view.setPointerCapture(event.pointerId)
      event.target.dispatchEvent(new MouseEvent('mousedown', { button: 2 }))
    })
    view.addEventListener('pointermove', event => {
      if (event.pointerId !== look?.id) return
      window.dispatchEvent(new MouseEvent('mousemove', {
        movementX: event.clientX - look.x,
        movementY: event.clientY - look.y,
      }))
      look.x = event.clientX
      look.y = event.clientY
    })
    for (const type of ['pointerup', 'pointercancel', 'lostpointercapture']) {
      view.addEventListener(type, event => { if (event.pointerId === look?.id) stopLook() })
    }
    window.addEventListener('blur', releaseAll)
    document.addEventListener('visibilitychange', () => { if (document.hidden) releaseAll() })
  }, { once: true })
})()
