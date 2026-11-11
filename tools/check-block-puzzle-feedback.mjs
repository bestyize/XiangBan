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
    this.hidden = id === 'continueButton'
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

class FakeMedia extends FakeElement {
  paused = true
  duration = .8
  playCount = 0
  play() {
    this.paused = false
    this.playCount += 1
    return Promise.resolve()
  }
  pause() { this.paused = true }
}

class FakeAudioParam {
  setValueAtTime(value) { this.value = value }
  linearRampToValueAtTime(peak) { this.peak = peak }
  exponentialRampToValueAtTime(end) { this.end = end }
  setTargetAtTime(target) { this.target = target }
}

function fakeAudioContext(graph) {
  const node = () => ({ connect(next) { return next }, disconnect() {}, start() {}, stop() {} })
  return class {
    state = 'running'
    currentTime = 1
    sampleRate = 44100
    destination = node()
    createGain() {
      const gain = new FakeAudioParam()
      graph.gains.push(gain)
      return { ...node(), gain }
    }
    createOscillator() {
      const frequency = new FakeAudioParam()
      graph.pitches.push(frequency)
      return { ...node(), frequency }
    }
    createBuffer(_channels, length) { return { getChannelData: () => new Float32Array(length) } }
    createBufferSource() { return node() }
    createBiquadFilter() { return { ...node(), Q: {}, frequency: new FakeAudioParam() } }
  }
}

function scenario(board, combo = 0, drought = 0, options = {}) {
  const nodes = new Map()
  const buttons = [new FakeElement(), new FakeElement(), new FakeElement()]
  const timers = []
  const audioGraph = { gains: [], pitches: [] }
  let computedColorReads = 0
  const save = {
    version: 2,
    game: { board, hand: ['dot', 'dot', 'dot'], score: 10,
      lines: 0, combo, drought, setupMoves: options.setupMoves || 0, bestCombo: combo, deals: 1 },
    best: 20, soundEnabled: options.soundEnabled ?? false, paletteIndex: 0, perfectCount: 0,
  }
  const storage = new Map(options.hostSave ? [] : [['xiaoban.block-puzzle.classic.v2', JSON.stringify(save)]])
  let releaseHostSave = null
  const hostedStorage = new Map(options.hostSave
    ? [['xiaoban.block-puzzle.classic.v2', options.hostSave]] : [])
  const xb = options.hostSave ? {
    info: { platform: 'android' },
    ready() { return Promise.resolve() },
    on() {},
    game: { start: async () => {}, progress: async () => {}, result: async () => {} },
    storage: {
      get(key) {
        if (key === 'xiaoban.block-puzzle.classic.v2' && options.deferHost) {
          return new Promise(resolve => { releaseHostSave = () => resolve(hostedStorage.get(key)) })
        }
        return Promise.resolve(hostedStorage.get(key) ?? null)
      },
      set(key, value) { hostedStorage.set(key, value); return Promise.resolve() },
      remove(key) { hostedStorage.delete(key); return Promise.resolve() },
    },
  } : undefined
  if (options.theme) storage.set('xiaoban.block-puzzle.theme.v1', JSON.stringify(options.theme))
  const document = {
    body: new FakeElement('body'),
    activeElement: null,
    getElementById(id) {
      if (!nodes.has(id)) nodes.set(id, id === 'musicTrack' || id.startsWith('voice') ? new FakeMedia(id) : new FakeElement(id))
      return nodes.get(id)
    },
    querySelectorAll(selector) { return selector === '[data-slot]' ? buttons : [] },
    createElement() { return new FakeElement() },
    addEventListener() {},
  }
  const window = {
    AudioContext: fakeAudioContext(audioGraph),
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
    window, document, URLSearchParams, xb,
    navigator: {},
    performance: { now: () => 100 },
    localStorage: {
      getItem(key) { return storage.get(key) ?? null },
      setItem(key, value) { storage.set(key, value) },
      removeItem(key) { storage.delete(key) },
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
  const initial = {
    board: nodes.get('board').children.map(row => row.children.map(cell => cell.className)),
    score: nodes.get('score').textContent,
    best: nodes.get('best').textContent,
    canResume: !nodes.get('continueButton').hidden,
  }
  // Existing clear/score tests resume an explicit saved scenario, whereas
  // default-entry tests below verify the real player-facing blank board.
  if (options.autoContinue !== false && !nodes.get('continueButton').hidden) {
    nodes.get('continueButton').handlers.click()
  }
  const placeAt = (row, col, slot = 0) => {
    buttons[slot].handlers.click()
    nodes.get('board').children[row].children[col].handlers.click()
  }
  const placeAtTopLeft = () => placeAt(0, 0)
  const saved = () => JSON.parse(storage.get('xiaoban.block-puzzle.classic.v2'))
  const savedTheme = () => JSON.parse(storage.get('xiaoban.block-puzzle.theme.v1'))
  const restart = () => nodes.get('restartOverlay').handlers.click()
  return {
    nodes, document, timers, audioGraph, initial, placeAt, placeAtTopLeft, saved, savedTheme, restart,
    getComputedColorReads: () => computedColorReads,
    releaseHostSave: () => releaseHostSave?.(),
    getHosted: key => hostedStorage.get(key),
  }
}

// Opening the app must show a blank 8x8 board, even with a saved partial game.
const previous = Array.from({ length: 8 }, () => Array(8).fill(0))
previous[2][3] = 1
previous[2][4] = 2
const freshStart = scenario(previous, 0, 0, { autoContinue: false })
assert.equal(freshStart.initial.score, 0)
assert.equal(freshStart.initial.best, 20)
assert.ok(freshStart.initial.board.flat().every(name => name === 'cell'))
assert.equal(freshStart.initial.canResume, true)
assert.equal(freshStart.nodes.get('overlay').hidden, true)
assert.equal(freshStart.saved().game.score, 0)
assert.ok(freshStart.nodes.get('continueButton').hidden === false)
freshStart.nodes.get('continueButton').handlers.click()
assert.equal(freshStart.saved().game.score, 10)
assert.equal(freshStart.saved().game.board[2][3], 1)
assert.equal(freshStart.saved().game.board[2][4], 2)
assert.equal(freshStart.nodes.get('continueButton').hidden, true)

// Android may return the last round after the player has already made a move.
// That delayed read may offer continuation, but must NEVER swap the live board.
const remoteBoard = Array.from({ length: 8 }, () => Array(8).fill(0))
remoteBoard[6][6] = 1
const remoteSave = {
  version: 2,
  game: { board: remoteBoard, hand: ['dot', 'dot', 'dot'], score: 40,
    lines: 0, combo: 0, drought: 0, bestCombo: 0, deals: 1 },
  best: 60, soundEnabled: true, paletteIndex: 0, perfectCount: 0,
}
const asyncStart = scenario(remoteBoard, 0, 0,
  { hostSave: remoteSave, deferHost: true, autoContinue: false })
assert.equal(asyncStart.initial.score, 0)
asyncStart.placeAt(0, 0)
const scoreAfterMove = asyncStart.saved().game.score
assert.ok(scoreAfterMove > 0)
asyncStart.releaseHostSave()
for (let i = 0; i < 18; i++) await Promise.resolve()
assert.equal(asyncStart.saved().game.score, scoreAfterMove)
assert.equal(asyncStart.saved().game.board[6][6], 0)
assert.equal(asyncStart.saved().game.board[0][0] !== 0, true)
assert.equal(asyncStart.nodes.get('overlay').hidden, true)

// Cross clears must produce an orderly horizontal AND vertical beam, and
// consecutive clears should use a louder reward tier rather than random sparks.
const crossBoard = Array.from({ length: 8 }, () => Array(8).fill(0))
for (let index = 1; index < 8; index++) {
  crossBoard[0][index] = 1
  crossBoard[index][0] = 1
}
crossBoard[5][5] = 1
const cross = scenario(crossBoard, 2, 0, { soundEnabled: true })
const track = cross.nodes.get('musicTrack')
const regularVolume = track.volume
await track.play()
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
assert.equal(track.volume, regularVolume, 'clears and callouts should keep the music level steady')
assert.equal(cross.nodes.get('voiceGood').playCount, 1)
assert.ok(cross.audioGraph.pitches.length > 5, 'a cross and combo should schedule several pitches')
cross.restart()
assert.equal(cross.audioGraph.gains[0].target, 0, 'restart should fade the previous combo cue')
assert.equal(cross.nodes.get('voiceGood').paused, true, 'restart should stop the previous callout')
assert.equal(track.volume, regularVolume)

// Single rows climb and single columns descend. A spoken cue is reserved for a reward.
const soloRow = crossBoard.map((row, i) => row.map((value, j) => i > 0 && j === 0 ? 0 : value))
const soloCol = crossBoard.map((row, i) => row.map((value, j) => i === 0 && j > 0 ? 0 : value))
const rowClear = scenario(soloRow, 0, 0, { soundEnabled: true })
const colClear = scenario(soloCol, 0, 0, { soundEnabled: true })
rowClear.placeAtTopLeft()
colClear.placeAtTopLeft()
assert.ok(rowClear.audioGraph.pitches[1].value < rowClear.audioGraph.pitches[3].value)
assert.ok(colClear.audioGraph.pitches[1].value > colClear.audioGraph.pitches[3].value)
assert.equal(rowClear.nodes.get('voiceNice').playCount, 0)

const nice = scenario(crossBoard, 0, 0, { soundEnabled: true })
nice.placeAtTopLeft()
assert.equal(nice.nodes.get('voiceNice').playCount, 1)
const great = scenario(soloRow, 4, 0, { soundEnabled: true })
great.placeAtTopLeft()
assert.equal(great.nodes.get('voiceGreat').playCount, 1)

// A true perfect clear switches the whole game's palette and persists the
// unlock. It must not rotate the palette on a normal line clear.
const perfectBoard = Array.from({ length: 8 }, () => Array(8).fill(0))
for (let index = 1; index < 8; index++) perfectBoard[0][index] = 1
const perfect = scenario(perfectBoard, 2, 0, { soundEnabled: true })
perfect.placeAtTopLeft()
assert.equal(perfect.nodes.get('voiceWonderful').playCount, 1)
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
assert.equal(finished.nodes.get('overlay').hidden, true)
assert.equal(finished.initial.score, 0)
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

console.log('PASS block-puzzle feedback: clean start, explicit continue, colored departures and combo')
