import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { describe, expect, it } from 'vitest'

type Listener = (event: Record<string, any>) => void

// Drive the shipped controller with DOM/pointer adapters; simulation and controller are not duplicated.
function runtime({ portrait = false, fullscreen = false, standalone = false, search = '?hostNonce=test-round' } = {}) {
  class Node {
    hidden = false
    disabled = false
    textContent: any = ''
    width = 844
    height = 390
    offsetWidth = 96
    dataset: Record<string, string> = {}
    attrs = new Map<string, string>()
    captures = new Set<number>()
    listeners = new Map<string, Listener[]>()
    classes = new Set<string>()
    children: Node[] = []
    cssProperties = new Map<string, any>()
    style = { left: '', top: '', bottom: '', width: '', height: '', transform: '', opacity: '', background: '', setProperty: (key: string, value: any) => { this.cssProperties.set(key, value) } }
    classList = { add: (value: string) => this.classes.add(value), remove: (value: string) => this.classes.delete(value), toggle: (value: string, active: boolean) => active ? this.classes.add(value) : this.classes.delete(value) }
    id: string
    constructor(id = '') { this.id = id }
    addEventListener(type: string, listener: Listener) { this.listeners.set(type, [...(this.listeners.get(type) || []), listener]) }
    fire(type: string, event: Record<string, any> = {}) {
      for (const listener of this.listeners.get(type) || []) listener({ preventDefault() {}, button: 0, ...event })
    }
    setAttribute(name: string, value: string) { this.attrs.set(name, value) }
    setPointerCapture(id: number) { this.captures.add(id) }
    hasPointerCapture(id: number) { return this.captures.has(id) }
    releasePointerCapture(id: number) { this.captures.delete(id) }
    getBoundingClientRect() {
      const rotation = Number(nodes.get('game')?.dataset.rotation || 0)
      let left = 30, top = 260
      if (this.id === 'joystick' && rotation === 1) { left = win.innerWidth - 260 - 96; top = 30 }
      if (this.id === 'joystick' && rotation === -1) { left = 260; top = win.innerHeight - 30 - 96 }
      return { x: left, y: top, top, left, width: this.hidden ? 0 : 96, height: 96 }
    }
    append(...children: Node[]) { this.children.push(...children) }
    replaceChildren(...children: Node[]) { this.children = children }
    querySelectorAll() { return [nodes.get('resumeButton')!, nodes.get('restartButton')!] }
    focus() { doc.activeElement = this }
    blur() { doc.activeElement = null }
    getContext() { return drawing }
  }
  const drawing = new Proxy({ createLinearGradient: () => ({ addColorStop() {} }), createRadialGradient: () => ({ addColorStop() {} }), createPattern: () => ({}) }, {
    get(target, key) { return key in target ? target[key as keyof typeof target] : () => {} },
  })
  const nodes = new Map<string, Node>()
  const node = (id: string) => {
    if (!nodes.has(id)) nodes.set(id, new Node(id))
    return nodes.get(id)!
  }
  const modes = [new Node('cozy'), new Node('lively')]
  modes[0].dataset.mode = 'cozy'; modes[1].dataset.mode = 'lively'
  const roundChoices = [node('timedModeButton'), node('endlessModeButton')]
  roundChoices[0].dataset.roundType = 'timed'; roundChoices[1].dataset.roundType = 'endless'
  const messages: Record<string, any>[] = []
  const navigations: string[] = []
  const doc = Object.assign(new Node('document'), {
    hidden: false, activeElement: null as Node | null, documentElement: new Node('root'), fullscreenEnabled: fullscreen,
    getElementById: node, querySelector: (selector: string) => node(selector),
    querySelectorAll: (selector: string) => selector === '[data-round-type]' ? roundChoices : modes, createElement: (tag: string) => new Node(tag),
    exitFullscreen: () => Promise.resolve(),
  })
  const win = Object.assign(new Node('window'), {
    innerWidth: portrait ? 390 : 844, innerHeight: portrait ? 844 : 390, devicePixelRatio: 3,
    parent: { postMessage: (message: Record<string, any>) => messages.push(message) },
    location: { search, assign: (url: string) => { navigations.push(url) } },
  })
  if (standalone) win.parent = win as any
  let frame: ((time: number) => void) | null = null, now = 0
  const context: Record<string, any> = {
    window: win, document: doc, location: win.location, screen: { orientation: { unlock() {} } },
    URLSearchParams, HTMLButtonElement: Node,
    matchMedia: (query: string) => ({ matches: query === '(pointer: coarse)' }),
    getComputedStyle: (element: Node) => ({ getPropertyValue: (name: string) => element.cssProperties.get(name) || '0px' }),
    requestAnimationFrame: (callback: (time: number) => void) => { frame = callback },
    setTimeout: () => 1, clearTimeout() {}, performance: { now: () => now },
  }
  runInNewContext(readFileSync(new URL('../../public/game-packs/snake-arena/rules.js', import.meta.url), 'utf8'), context)
  runInNewContext(readFileSync(new URL('../../public/game-packs/snake-arena/viewport.js', import.meta.url), 'utf8'), context)
  let model: any
  const original = context.XiabanSnakeArena.createGame
  context.XiabanSnakeArena.createGame = (options: any) => { model = original({ seed: 42, ...options }); return model }
  runInNewContext(readFileSync(new URL('../../public/game-packs/snake-arena/game.js', import.meta.url), 'utf8'), context)
  function advance(count: number) { for (let i = 0; i < count; i += 1) { now += 1000 / 60; frame!(now) } }
  function init(overrides: Record<string, any> = {}) {
    win.fire('message', { source: win.parent, data: { channel: 'xiaban.game-host', protocol: 1, nonce: 'test-round', type: 'host.init', ...overrides } })
  }
  function start() { node('startButton').fire('click'); model.snakes = [model.snakes[0]]; model.food = []; advance(150) }
  return { node, doc, win, init, messages, navigations, advance, start, difficultyChoices: modes, state: () => model }
}

describe('snake arena mobile controller and host protocol', () => {
  it('shares only completed rounds and does not add another result or score update', () => {
    const game = runtime()
    game.init(); game.start()
    game.node('shareResultButton').fire('click')
    expect(game.messages.some(message => message.type === 'game.share')).toBe(false)
    game.state().snakes[0].x = 2399; game.state().snakes[0].score = 450
    game.advance(3)
    const count = game.messages.length
    game.node('shareResultButton').fire('click')
    expect(game.messages[count].type).toBe('game.share')
    expect(game.messages[count].result).toMatchObject({ score: 450, roundType: 'timed', difficulty: 'cozy' })
    expect(game.messages.filter(message => message.type === 'game.result')).toHaveLength(1)
    game.node('restartButton').fire('click')
    const shared = game.messages.filter(message => message.type === 'game.share').length
    game.node('shareResultButton').fire('click')
    expect(game.messages.filter(message => message.type === 'game.share')).toHaveLength(shared)
  })
  it('opens the public result-page route when the pack is played directly', () => {
    const game = runtime({ standalone: true })
    game.start()
    game.state().snakes[0].x = 2399
    game.advance(3); game.node('shareResultButton').fire('click')
    expect(game.navigations).toHaveLength(1)
    expect(game.navigations[0]).toMatch(/^\/games\/snake-arena\/share\?own=1#v=1&/)
    expect(game.navigations[0]).not.toContain('hostNonce')
  })
  it('starts a friend challenge with the original round type and difficulty', () => {
    const game = runtime({ search: '?hostNonce=test-round&round=endless&difficulty=lively' })
    game.start()
    expect(game.state().roundType).toBe('endless')
    expect(game.state().mode).toBe('lively')
    expect(game.node('endlessModeButton').attrs.get('aria-pressed')).toBe('true')
    expect(game.difficultyChoices[1].attrs.get('aria-pressed')).toBe('true')
  })
  it('starts endless rounds with an elapsed clock and no automatic result after three minutes', () => {
    const game = runtime({ portrait: true })
    game.init()
    game.node('endlessModeButton').fire('click')
    expect(game.node('roundSummary').textContent).toBe('不限时，活着就继续')
    game.start()
    game.state().time = 180.1
    game.advance(20)
    expect(game.state().roundType).toBe('endless')
    expect(game.state().phase).toBe('playing')
    expect(game.node('clockLabel').textContent).toBe('无尽生存')
    expect(game.node('time').textContent).toBe('3:00')
    expect(game.node('.clock').classes.has('urgent')).toBe(false)
    expect(game.messages.filter(message => message.type === 'game.result')).toHaveLength(0)
    game.doc.hidden = true; game.doc.fire('visibilitychange')
    const time = game.state().time
    game.advance(30)
    expect(game.state().time).toBe(time)
  })

  it('retains endless mode and difficulty on restart and posts results only on elimination', () => {
    const game = runtime()
    game.init()
    game.node('endlessModeButton').fire('click')
    game.difficultyChoices[1].fire('click')
    game.start()
    expect(game.state().mode).toBe('lively')
    expect(game.node('endlessModeButton').attrs.get('aria-pressed')).toBe('true')
    game.state().snakes[0].x = 2399; game.state().snakes[0].score = 450
    game.advance(3)
    expect(game.state().reason).toBe('wall')
    expect(game.node('resultBadge').textContent).toContain('无尽生存')
    const results = game.messages.filter(message => message.type === 'game.result')
    expect(results).toHaveLength(1)
    expect(results[0].score).toBe(450)
    game.node('restartButton').fire('click')
    expect(game.state().roundType).toBe('endless')
    expect(game.state().remaining).toBeNull()
    expect(game.state().mode).toBe('lively')
    expect(game.node('clockLabel').textContent).toBe('无尽生存')
    expect(game.node('time').textContent).toBe('0:00')
    expect(game.messages.filter(message => message.type === 'game.start')).toHaveLength(2)
  })

  it('switches back to timed competition independently of the difficulty', () => {
    const game = runtime()
    game.node('endlessModeButton').fire('click')
    game.difficultyChoices[1].fire('click')
    game.node('timedModeButton').fire('click')
    game.start()
    expect(game.state().roundType).toBe('timed')
    expect(game.state().mode).toBe('lively')
    expect(game.state().remaining).not.toBeNull()
    expect(game.node('clockLabel').textContent).toBe('三分钟竞技')
  })
  it('applies verified host safe areas and updates them after rotation without starting a round', () => {
    const game = runtime()
    game.init({ safeArea: { top: 0, right: 47, bottom: 21, left: 47 } })
    expect(game.node('game').cssProperties.get('--safe-right')).toBe('47px')
    game.init({ type: 'host.layout', safeArea: { right: 0, left: 0, bottom: 34, top: -10 } })
    expect(game.node('game').cssProperties.get('--safe-bottom')).toBe('34px')
    expect(game.node('game').cssProperties.get('--safe-top')).toBe('0px')
    expect(game.messages.filter((message) => message.type === 'game.start')).toHaveLength(0)
  })
  it('acknowledges readiness without creating a phantom welcome-screen round', () => {
    const game = runtime()
    game.init({ nonce: 'wrong' })
    expect(game.messages).toHaveLength(0)
    game.init()
    expect(game.messages.map((message) => message.type)).toEqual(['game.ready'])
    game.start()
    expect(game.messages.filter((message) => message.type === 'game.start')).toHaveLength(1)
    game.init()
    expect(game.messages.filter((message) => message.type === 'game.start')).toHaveLength(1)
  })

  it('reports a round once after late host initialization, and a new round on restart', () => {
    const game = runtime()
    game.start(); game.init(); game.init()
    expect(game.messages.filter((message) => message.type === 'game.start')).toHaveLength(1)
    game.node('restartButton').fire('click')
    expect(game.messages.filter((message) => message.type === 'game.start')).toHaveLength(2)
  })

  it('supports two simultaneous fingers for floating steering and boost, then clears canceled inputs', () => {
    const game = runtime()
    game.start()
    const player = game.state().snakes[0]
    const originalAngle = player.angle
    game.node('arena').fire('pointerdown', { pointerId: 1, pointerType: 'touch', clientX: 180, clientY: 270 })
    game.node('arena').fire('pointermove', { pointerId: 1, pointerType: 'touch', clientX: 180, clientY: 210 })
    game.node('boostButton').fire('pointerdown', { pointerId: 2, pointerType: 'touch', clientX: 775, clientY: 305 })
    game.advance(15)
    expect(player.angle).toBeLessThan(originalAngle - 0.3)
    expect(player.boosting).toBe(true)
    expect(game.node('arena').captures.has(1)).toBe(true)
    expect(game.node('boostButton').captures.has(2)).toBe(true)
    game.node('arena').fire('pointercancel', { pointerId: 1 })
    game.node('boostButton').fire('pointercancel', { pointerId: 2 })
    game.advance(2)
    expect(player.boosting).toBe(false)
    expect(game.node('stick').style.transform).toBe('')
    expect(game.node('boostButton').attrs.get('aria-pressed')).toBe('false')
  })

  it('stops the simulation on portrait rotation and waits for an explicit resume after rotating back', () => {
    const game = runtime()
    game.start()
    game.win.innerWidth = 390; game.win.innerHeight = 844; game.win.fire('resize')
    const time = game.state().time
    expect(game.node('game').dataset.rotation).toBe('1')
    expect(game.node('pauseOverlay').hidden).toBe(false)
    game.advance(120)
    expect(game.state().time).toBe(time)
    game.win.innerWidth = 844; game.win.innerHeight = 390; game.win.fire('resize')
    expect(game.node('game').dataset.rotation).toBe('0')
    game.advance(30)
    expect(game.state().time).toBe(time)
    game.node('resumeButton').fire('click'); game.advance(30)
    expect(game.state().time).toBeGreaterThan(time)
  })

  it('pauses on backgrounding and releases both captured pointers to prevent stuck acceleration', () => {
    const game = runtime()
    game.start()
    game.node('joystick').fire('pointerdown', { pointerId: 1, pointerType: 'touch', clientX: 80, clientY: 305 })
    game.node('boostButton').fire('pointerdown', { pointerId: 2, pointerType: 'touch' })
    game.doc.hidden = true; game.doc.fire('visibilitychange')
    const time = game.state().time
    game.advance(120)
    expect(game.state().time).toBe(time)
    expect(game.node('joystick').captures.size).toBe(0)
    expect(game.node('boostButton').captures.size).toBe(0)
    game.doc.hidden = false; game.node('resumeButton').fire('click'); game.advance(2)
    expect(game.state().snakes[0].boosting).toBe(false)
  })

  it('plays immediately in a portrait browser when fullscreen and orientation locking are unavailable', () => {
    const game = runtime({ portrait: true })
    expect(game.node('game').dataset.rotation).toBe('1')
    expect(game.node('game').style.width).toBe('844px')
    expect(game.node('game').style.height).toBe('390px')
    expect(game.node('fullscreenButton').hidden).toBe(true)
    game.start()
    expect(game.state().time).toBeGreaterThan(0)
    expect(game.node('pauseOverlay').hidden).toBe(true)
    expect(game.node('arena').width).toBe(844 * 2)
  })

  it.each([
    { rotation: 1, startX: 120, startY: 180, endX: 180, endY: 180, boostX: 85, boostY: 775 },
    { rotation: -1, startX: 270, startY: 664, endX: 210, endY: 664, boostX: 305, boostY: 69 },
  ])('maps two-finger steering and boost correctly in rotation $rotation', (gesture) => {
    const game = runtime({ portrait: true })
    if (gesture.rotation === -1) game.node('welcomeFlip').fire('click')
    game.start()
    const player = game.state().snakes[0], angle = player.angle
    expect(game.node('game').dataset.rotation).toBe(String(gesture.rotation))
    game.node('arena').fire('pointerdown', { pointerId: 1, pointerType: 'touch', clientX: gesture.startX, clientY: gesture.startY })
    game.node('arena').fire('pointermove', { pointerId: 1, pointerType: 'touch', clientX: gesture.endX, clientY: gesture.endY })
    game.node('boostButton').fire('pointerdown', { pointerId: 2, pointerType: 'touch', clientX: gesture.boostX, clientY: gesture.boostY })
    game.advance(15)
    expect(player.angle).toBeLessThan(angle - 0.3)
    expect(player.boosting).toBe(true)
    expect(game.node('joystick').style.left).toBe('132px')
    expect(game.node('joystick').style.top).toBe('222px')
    game.node('boostButton').fire('pointercancel', { pointerId: 2 })
    game.advance(2)
    expect(player.boosting).toBe(false)
  })

  it.each([
    { rotation: 1, startX: 82, startY: 78, endX: 142, endY: 78 },
    { rotation: -1, startX: 308, startY: 766, endX: 248, endY: 766 },
  ])('converts the fixed joystick center correctly in rotation $rotation', (gesture) => {
    const game = runtime({ portrait: true })
    if (gesture.rotation === -1) game.node('welcomeFlip').fire('click')
    game.start()
    const player = game.state().snakes[0], angle = player.angle
    game.node('joystick').fire('pointerdown', { pointerId: 1, pointerType: 'touch', clientX: gesture.startX, clientY: gesture.startY })
    game.node('joystick').fire('pointermove', { pointerId: 1, pointerType: 'touch', clientX: gesture.endX, clientY: gesture.endY })
    game.advance(15)
    expect(player.angle).toBeLessThan(angle - 0.3)
  })

  it('remaps host safe areas for both rotation directions', () => {
    const game = runtime({ portrait: true })
    game.init({ safeArea: { top: 44, right: 0, bottom: 34, left: 0 } })
    expect(game.node('game').cssProperties.get('--safe-left')).toBe('44px')
    expect(game.node('game').cssProperties.get('--safe-right')).toBe('34px')
    game.node('welcomeFlip').fire('click')
    expect(game.node('game').cssProperties.get('--safe-top')).toBe('0px')
    expect(game.node('game').cssProperties.get('--safe-left')).toBe('34px')
    expect(game.node('game').cssProperties.get('--safe-right')).toBe('44px')
  })

  it('pauses a running round when flipping and releases captured fingers before resuming', () => {
    const game = runtime({ portrait: true })
    game.start()
    game.node('arena').fire('pointerdown', { pointerId: 1, pointerType: 'touch', clientX: 120, clientY: 180 })
    game.node('boostButton').fire('pointerdown', { pointerId: 2, pointerType: 'touch' })
    game.node('flipButton').fire('click')
    expect(game.node('game').dataset.rotation).toBe('-1')
    expect(game.node('pauseOverlay').hidden).toBe(false)
    expect(game.node('arena').captures.size).toBe(0)
    expect(game.node('boostButton').captures.size).toBe(0)
    const time = game.state().time
    game.advance(60)
    expect(game.state().time).toBe(time)
    game.node('resumeButton').fire('click'); game.advance(2)
    expect(game.state().time).toBeGreaterThan(time)
    expect(game.state().snakes[0].boosting).toBe(false)
  })

  it('continues the initial countdown when native fullscreen changes the viewport orientation', () => {
    const game = runtime({ portrait: true })
    game.node('startButton').fire('click')
    game.state().snakes = [game.state().snakes[0]]; game.state().food = []
    game.win.innerWidth = 844; game.win.innerHeight = 390; game.win.fire('resize')
    expect(game.node('pauseOverlay').hidden).toBe(true)
    game.advance(180)
    expect(game.state().time).toBeGreaterThan(0)
  })

  it('submits a single final score and shows a playable restart on elimination', () => {
    const game = runtime()
    game.init(); game.start()
    const player = game.state().snakes[0]
    player.x = 2399; player.score = 450
    game.advance(3)
    const results = game.messages.filter((message) => message.type === 'game.result')
    expect(results).toHaveLength(1)
    expect(results[0].score).toBe(450)
    expect(game.node('resultOverlay').hidden).toBe(false)
    expect(game.node('controls').hidden).toBe(true)
    const progressCount = game.messages.filter((message) => message.type === 'game.progress').length
    game.init()
    expect(game.messages.filter((message) => message.type === 'game.result')).toHaveLength(1)
    expect(game.messages.filter((message) => message.type === 'game.progress')).toHaveLength(progressCount)
    game.win.fire('pagehide')
    expect(game.messages.filter((message) => message.type === 'game.progress')).toHaveLength(progressCount)
  })
})
