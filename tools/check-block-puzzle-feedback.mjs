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
    this.style = { setProperty() {}, removeProperty() {} }
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
  append(child) { this.children.push(child) }
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

function scenario(board, combo = 0, drought = 0) {
  const nodes = new Map()
  const buttons = [new FakeElement(), new FakeElement(), new FakeElement()]
  const timers = []
  const save = {
    version: 2,
    game: { board, hand: ['dot', 'dot', 'dot'], score: 10,
      lines: 0, combo, drought, bestCombo: combo, deals: 1 },
    best: 20, soundEnabled: false, paletteIndex: 0, perfectCount: 0,
  }
  const storage = new Map([['xiaoban.block-puzzle.classic.v2', JSON.stringify(save)]])
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
    clearTimeout() {},
  }
  runInNewContext(rulesText, sandbox, { filename: 'rules.js' })
  runInNewContext(gameText, sandbox, { filename: 'game.js' })
  const placeAtTopLeft = () => {
    buttons[0].handlers.click()
    nodes.get('board').children[0].children[0].handlers.click()
  }
  const saved = () => JSON.parse(storage.get('xiaoban.block-puzzle.classic.v2'))
  return { nodes, document, timers, placeAtTopLeft, saved }
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
assert.match(cross.nodes.get('comboBanner').textContent, /COMBO ×3/)
assert.equal(cross.saved().game.combo, 3)
assert.equal(cross.saved().paletteIndex, 0)
assert.ok(Number(cross.nodes.get('score').textContent) > 10)

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

const empty = Array.from({ length: 8 }, () => Array(8).fill(0))
const buffer = scenario(empty, 2, 0)
buffer.placeAtTopLeft()
assert.match(buffer.nodes.get('status').textContent, /连击保留/)
assert.equal(buffer.saved().game.combo, 2)
console.log('PASS block-puzzle feedback: aligned beams, combo tiers, perfect palette and grace moves')
