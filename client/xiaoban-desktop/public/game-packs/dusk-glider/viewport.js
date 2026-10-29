(() => {
  'use strict'

  function layout(physicalWidth, physicalHeight, preferredRotation = 1) {
    const rotation = physicalWidth < physicalHeight ? (preferredRotation < 0 ? -1 : 1) : 0
    return {
      physicalWidth, physicalHeight, rotation,
      width: rotation ? physicalHeight : physicalWidth,
      height: rotation ? physicalWidth : physicalHeight,
    }
  }

  function transform(view) {
    if (view.rotation === 1) return 'translate(' + view.physicalWidth + 'px, 0) rotate(90deg)'
    if (view.rotation === -1) return 'translate(0, ' + view.physicalHeight + 'px) rotate(-90deg)'
    return 'none'
  }

  function point(view, x, y) {
    if (view.rotation === 1) return { x: y, y: view.physicalWidth - x }
    if (view.rotation === -1) return { x: view.physicalHeight - y, y: x }
    return { x, y }
  }

  function safeArea(view, physical) {
    if (view.rotation === 1) return { top: physical.right, right: physical.bottom, bottom: physical.left, left: physical.top }
    if (view.rotation === -1) return { top: physical.left, right: physical.top, bottom: physical.right, left: physical.bottom }
    return { ...physical }
  }

  globalThis.XiabanGliderViewport = { layout, transform, point, safeArea }
})()
