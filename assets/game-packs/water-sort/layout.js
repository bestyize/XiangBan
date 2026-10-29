(() => {
  'use strict'
  const clamp = (value, min, max) => Math.max(min, Math.min(max, value))
  function getTubeLayout(count, width, height) {
    if (!Number.isInteger(count) || count < 1 || count > 10 || !Number.isFinite(width) || !Number.isFinite(height) || width < 80 || height < 100) return null
    const wide = width > height * 1.6 && width / count >= 48
    const columns = wide || count <= 5 ? count : Math.ceil(count / 2)
    const rows = Math.ceil(count / columns)
    const gap = clamp(Math.floor(width * .026), 6, 14)
    const rowGap = clamp(Math.floor(height * .04), 12, 24)
    const paddingX = width < 340 ? 10 : 14
    const paddingTop = 24, paddingBottom = 20
    const slotWidth = (width - paddingX * 2 - gap * (columns - 1)) / columns
    const fitWidth = Math.min(88, slotWidth - 8)
    // Each button reserves 24px for the gap, cup number and bottom padding.
    const availableHeight = (height - paddingTop - paddingBottom - rowGap * (rows - 1)) / rows - 24
    if (fitWidth < 1 || availableHeight < 1) return null
    const tubeHeight = Math.max(1, Math.floor(Math.min(280, fitWidth * 3.4, availableHeight)))
    const tubeWidth = Math.max(1, Math.floor(Math.min(fitWidth, tubeHeight / 3.1)))
    const lastRowCount = count - columns * (rows - 1)
    // Two CSS tracks per cup allow a half-slot offset to center an odd last row.
    const startColumns = Array.from({ length: count }, (_, i) => i % columns * 2 + 1 + (Math.floor(i / columns) === rows - 1 ? columns - lastRowCount : 0))
    return { columns, rows, tubeWidth, tubeHeight, gap, rowGap, paddingX, paddingTop, paddingBottom, startColumns }
  }
  globalThis.XiabanWaterSortLayout = { getTubeLayout }
})()
