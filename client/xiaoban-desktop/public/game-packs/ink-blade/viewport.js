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

  function combatFrame(player, opponents, width, left, right, marginLeft, marginRight) {
    const half = width / 2
    let min = player.right - half + marginRight
    let max = player.left + half - marginLeft

    for (const foe of opponents) {
      const a = Math.max(min, foe.right - half + marginRight)
      const b = Math.min(max, foe.left + half - marginLeft)
      if (a <= b) { min = a; max = b }
    }

    let safeMin = Math.max(min, player.right - half + right)
    let safeMax = Math.min(max, player.left + half - left)
    // Keep the most immediate opponents clear of the controls when the view has enough space.
    if (safeMin <= safeMax) {
      for (const foe of opponents) {
        const a = Math.max(safeMin, foe.right - half + right)
        const b = Math.min(safeMax, foe.left + half - left)
        if (a <= b) { safeMin = a; safeMax = b }
      }
    } else {
      safeMin = min; safeMax = max
    }
    return { min, max, safeMin, safeMax }
  }

  globalThis.XiabanInkViewport = { layout, transform, point, safeArea, combatFrame }
})()
