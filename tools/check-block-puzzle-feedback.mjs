import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { runInNewContext } from 'node:vm'

const rulesPath = fileURLToPath(new URL('../assets/game-packs/block-puzzle/rules.js', import.meta.url))
const gamePath = fileURLToPath(new URL('../assets/game-packs/block-puzzle/game.js', import.meta.url))
const rulesText = readFileSync(rulesPath, 'utf8')
const gameText = readFileSync(gamePath, 'utf8')

class FakeElement {
  constructor(id = '') {
    this.id = id
    this.children = []
    this.handlers = {}
    this.attributes = {}
    this.dataset = {}
    this.hidden = false
    this.disabled = false
    this.className = ''
    this.offsetWidth = 40
    this.cssVars = {}
    this.style = {
      setProperty: (key, value) => { this.cssVars[key] = value },
      removeProperty: key => { delete this.cssVars[key] },
    }
    this.classes = new Set()
    this.classList = {
      add: (...names) => names.forEach(name => this.classes.add(name)),
      remove: (...names) => names.forEach(name => this.classes.delete(name)),
      contains: name => this.classes.has(name),
      toggle: (name, enabled) => {
        if (enabled) this.classes.add(name)
        else this.classes.delete(name)
      },
    }
  }
  addEventListener(name, callback) { this.handlers[name] = callback }
  setAttribute(name, value) { this.attributes[name] = value }
  replaceChildren(...children) { this.children = children }
  append(...children) { this.children.push(...children) }
  get childElementCount() { return this.children.length }
  getBoundingClientRect() { return { top: 200, left: 0, width: 40, height: 40 } }
  focus() {}
  querySelector(selector) {
    this.subnodes ||= new Map()
    if (!this.subnodes.has(selector)) this.subnodes.set(selector, new FakeElement(selector))
    return this.subnodes.get(selector)
  }
  hasPointerCapture() { return false }
  releasePointerCapture() {}
  setPointerCapture() {}
}

function scenario(board, combo = 0, drought = 0, options = {}) {
  const nodes = new Map()
  const buttons = [new FakeElement(), new FakeElement(), new FakeElement()]
  const timers = []
  let computedColorReads = 0
  const save = {
    version: 2,
    game: { board, hand: ['dot', 'dot', 'dot'], score: 10,
      lines: 0, combo, drought, setupMoves: options.setupMoves || 0, bestCombo: combo, deals: 1 },
    best: 20, soundEnabled: false, paletteIndex: 0, perfectCount: 0,
  }
  const storage = new Map([['xiaoban.block-puzzle.classic.v2', JSON.stringify(save)]])
  if (options.theme) storage.set('xiaoban.block-puzzle.theme.v1', JSON.stringify(options.theme))
  const document = {
    body: new FakeElement('body'),
    activeElement: null,
    getElementById(id) {
      if (!nodes.has(id)) nodes.set(id, new FakeElement(id))
      return nodes.get(id)
    },
    querySelectorAll(selector) { return selector === '[data-slot]' ? buttons : [] },
    createElement() { return new FakeElement() },
    addEventListener() {},
  }
  const window = {
    location: { search: '', assign() {} },
    matchMedia() { return { matches: false } },
    getComputedStyle() {
      computedColorReads += 1
      return { getPropertyValue(name) { return name === '--color-1' ? '#f65b97' : '' } }
    },
    addEventListener() {},
  }
  window.parent = window
  const sandbox = {
    window, document, URLSearchParams,
    navigator: {},
    performance: { now: () => 100 },
    localStorage: {
      getItem(key) { return storage.get(key) ?? null },
      setItem(key, value) { storage.set(key, value) },
    },
    requestAnimationFrame(callback) { callback(2000); return 1 },
    cancelAnimationFrame() {},
    setTimeout(callback, delay) { timers.push({ callback, delay }); return timers.length },
    clearTimeout(handle) {
      if (timers[handle - 1]) timers[handle - 1].cancelled = true
    },
  }
  runInNewContext(rulesText, sandbox, { filename: 'rules.js' })
  runInNewContext(gameText, sandbox, { filename: 'game.js' })
  const placeAt = (row, col, slot = 0) => {
    buttons[slot].handlers.click()
    nodes.get('board').children[row].children[col].handlers.click()
  }
  const placeAtTopLeft = () => placeAt(0, 0)
  const saved = () => JSON.parse(storage.get('xiaoban.block-puzzle.classic.v2'))
  const savedTheme = () => JSON.parse(storage.get('xiaoban.block-puzzle.theme.v1'))
  const restart = () => nodes.get('restartOverlay').handlers.click()
  return {
    nodes, document, timers, placeAt, placeAtTopLeft, saved, savedTheme, restart,
    getComputedColorReads: () => computedColorReads,
  }
}

// Cross clears must produce an orderly horizontal AND vertical beam, and
// consecutive clears should use a louder reward tier rather than random sparks.
const crossBoard = Array.from({ length: 8 }, () => Array(8).fill(0))
for (let index = 1; index < 8; index++) {
  crossBoard[0][index] = 1
  crossBoard[index][0] = 1
}
crossBoard[5][5] = 1
const cross = scenario(crossBoard, 2)
cross.placeAtTopLeft()
const fx = cross.nodes.get('sparkLayer').children.map(child => child.className)
assert.ok(fx.includes('clear-beam horizontal'))
assert.ok(fx.includes('clear-beam vertical'))
assert.ok(fx.includes('clear-shockwave'))
assert.equal(fx.filter(name => name === 'departing-tile').length, 15)
assert.ok(cross.nodes.get('sparkLayer').children
  .filter(tile => tile.className === 'departing-tile')
  .every(tile => tile.cssVars['--piece-color'] === '#f65b97'))
assert.equal(cross.getComputedColorReads(), 1)
const rowTiles = cross.nodes.get('sparkLayer').children
  .filter(tile => tile.className === 'departing-tile').slice(0, 8)
const rowDelays = rowTiles.map(tile => Number(tile.cssVars['--delay'].replace('ms', '')))
assert.deepEqual(rowDelays.slice(0, 7), [75, 87, 99, 111, 123, 135, 147])
assert.equal(cross.nodes.get('rewardDiamond').hidden, false)
assert.match(cross.nodes.get('comboBanner').textContent, /COMBO ×3/)
assert.equal(cross.saved().game.combo, 3)
assert.equal(cross.saved().paletteIndex, 0)
assert.ok(Number(cross.nodes.get('score').textContent) > 10)
const pendingClearNotes = cross.timers.filter(timer => [55, 127, 199, 271].includes(timer.delay))
assert.ok(pendingClearNotes.length >= 2)
cross.restart()
assert.ok(pendingClearNotes.every(timer => timer.cancelled),
  'restarted games must cancel the previous combo melody')

// A true perfect clear switches the whole game's palette and persists the
// unlock. It must not rotate the palette on a normal line clear.
const perfectBoard = Array.from({ length: 8 }, () => Array(8).fill(0))
for (let index = 1; index < 8; index++) perfectBoard[0][index] = 1
const perfect = scenario(perfectBoard, 2)
perfect.placeAtTopLeft()
assert.equal(perfect.saved().paletteIndex, 1)
assert.equal(perfect.saved().perfectCount, 1)
assert.match(perfect.nodes.get('comboBanner').textContent, /PERFECT!/)
assert.equal(perfect.document.body.dataset.palette, 'rose')
const paletteChange = perfect.timers.find(timer => timer.delay === 330)
assert.ok(paletteChange, 'theme rotation should be timed after the clear highlight')
paletteChange.callback()
assert.equal(perfect.document.body.dataset.palette, 'amber')
assert.equal(perfect.savedTheme().paletteIndex, 1)
perfect.restart()
assert.equal(perfect.document.body.dataset.palette, 'amber')
assert.equal(perfect.saved().paletteIndex, 1)
// An independent theme save beats an older round snapshot.
const reopened = scenario(Array.from({ length: 8 }, () => Array(8).fill(0)), 0, 0,
  { theme: { paletteIndex: 1, perfectCount: 1 } })
assert.equal(reopened.document.body.dataset.palette, 'amber')
assert.equal(reopened.savedTheme().perfectCount, 1)
// The same palette persists through a game-over screen and its restart.
const finished = scenario(Array.from({ length: 8 }, () => Array(8).fill(1)), 0, 0,
  { theme: { paletteIndex: 1, perfectCount: 1 } })
assert.equal(finished.nodes.get('overlay').hidden, false)
finished.restart()
assert.equal(finished.document.body.dataset.palette, 'amber')

const empty = Array.from({ length: 8 }, () => Array(8).fill(0))
const buffer = scenario(empty, 2, 0)
buffer.placeAtTopLeft()
assert.match(buffer.nodes.get('status').textContent, /连击保留/)
assert.equal(buffer.saved().game.combo, 2)
// Non-clearing milestones are scored and animated separately from Combo.
const setupBoard = Array.from({ length: 8 }, () => Array(8).fill(0))
for (let i = 0; i < 6; i++) setupBoard[0][i] = 1
const setup = scenario(setupBoard)
setup.placeAt(0, 6)
assert.equal(setup.saved().game.score, 19)
assert.equal(setup.saved().game.combo, 0)
assert.equal(setup.saved().game.lines, 0)
assert.equal(setup.saved().game.setupMoves, 1)
assert.match(setup.nodes.get('status').textContent, /布局奖励/)
assert.match(setup.nodes.get('comboBanner').textContent, /NICE SETUP!/)
assert.equal(setup.nodes.get('rewardDiamond').textContent, '+8')
assert.equal(setup.nodes.get('rewardDiamond').hidden, false)
assert.ok(setup.nodes.get('sparkLayer').children.every(tile => tile.className !== 'departing-tile'))
const milestone = scenario(empty, 0, 3, { setupMoves: 2 })
milestone.placeAtTopLeft()
assert.equal(milestone.saved().game.score, 16)
assert.match(milestone.nodes.get('comboBanner').textContent, /KEEP GOING!/)
assert.equal(milestone.saved().game.lines, 0)

console.log('PASS block-puzzle feedback: colored departures, quiet rewards, theme inheritance and combo')
