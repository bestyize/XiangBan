import { readFileSync } from 'node:fs'
import { runInNewContext } from 'node:vm'
import { describe, expect, it } from 'vitest'

type Listener = (event: Record<string, any>) => void

// Drive the shipped controller with DOM/pointer adapters; simulation and controller are not duplicated.
function runtime({ portrait = false, fullscreen = false } = {}) {
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
    style = { left: '', top: '', bottom: '', transform: '', opacity: '', background: '', setProperty: (key: string, value: any) => { this.cssProperties.set(key, value) } }
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
    getBoundingClientRect() { return { x: 30, y: 260, top: 260, left: 30, width: this.hidden ? 0 : 96, height: 96 } }
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
  const messages: Record<string, any>[] = []
  const doc = Object.assign(new Node('document'), {
    hidden: false, activeElement: null as Node | null, documentElement: new Node('root'), fullscreenEnabled: fullscreen,
    getElementById: node, querySelector: (selector: string) => node(selector),
    querySelectorAll: (_selector: string) => modes, createElement: (tag: string) => new Node(tag),
    exitFullscreen: () => Promise.resolve(),
  })
  const win = Object.assign(new Node('window'), {
    innerWidth: portrait ? 390 : 844, innerHeight: portrait ? 844 : 390, devicePixelRatio: 3,
    parent: { postMessage: (message: Record<string, any>) => messages.push(message) },
    location: { search: '?hostNonce=test-round', assign: () => {} },
  })
  let frame: ((time: number) => void) | null = null, now = 0
  const context: Record<string, any> = {
    window: win, document: doc, location: win.location, screen: { orientation: { unlock() {} } },
    URLSearchParams, HTMLButtonElement: Node,
    matchMedia: (query: string) => ({ matches: query === '(pointer: coarse)' }),
    requestAnimationFrame: (callback: (time: number) => void) => { frame = callback },
    setTimeout: () => 1, clearTimeout() {}, performance: { now: () => now },
  }
  runInNewContext(readFileSync(new URL('../../public/game-packs/snake-arena/rules.js', import.meta.url), 'utf8'), context)
  let model: any
  const original = context.XiabanSnakeArena.createGame
  context.XiabanSnakeArena.createGame = (options: any) => { model = original({ seed: 42, ...options }); return model }
  runInNewContext(readFileSync(new URL('../../public/game-packs/snake-arena/game.js', import.meta.url), 'utf8'), context)
  function advance(count: number) { for (let i = 0; i < count; i += 1) { now += 1000 / 60; frame!(now) } }
  function init(overrides: Record<string, any> = {}) {
    win.fire('message', { source: win.parent, data: { channel: 'xiaban.game-host', protocol: 1, nonce: 'test-round', type: 'host.init', ...overrides } })
  }
  function start() { node('startButton').fire('click'); model.snakes = [model.snakes[0]]; model.food = []; advance(150) }
  return { node, doc, win, init, messages, advance, start, state: () => model }
}

describe('snake arena mobile controller and host protocol', () => {
  it('applies verified host safe areas and updates them after rotation without starting a round', () => {
    const game = runtime()
    game.init({ safeArea: { top: 0, right: 47, bottom: 21, left: 47 } })
    expect(game.doc.documentElement.cssProperties.get('--safe-right')).toBe('max(env(safe-area-inset-right, 0px), 47px)')
    game.init({ type: 'host.layout', safeArea: { right: 0, left: 0, bottom: 34, top: -10 } })
    expect(game.doc.documentElement.cssProperties.get('--safe-bottom')).toBe('max(env(safe-area-inset-bottom, 0px), 34px)')
    expect(game.doc.documentElement.cssProperties.get('--safe-top')).toBe('max(env(safe-area-inset-top, 0px), 0px)')
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
    expect(game.node('rotateOverlay').hidden).toBe(false)
    expect(game.node('pauseOverlay').hidden).toBe(false)
    game.advance(120)
    expect(game.state().time).toBe(time)
    game.win.innerWidth = 844; game.win.innerHeight = 390; game.win.fire('resize')
    expect(game.node('rotateOverlay').hidden).toBe(true)
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

  it('keeps manual landscape play available when the fullscreen API is unavailable', () => {
    const game = runtime({ portrait: true })
    expect(game.node('rotateOverlay').hidden).toBe(false)
    expect(game.node('rotateFullscreen').hidden).toBe(true)
    game.win.innerWidth = 844; game.win.innerHeight = 390; game.win.fire('resize')
    game.start()
    expect(game.node('rotateOverlay').hidden).toBe(true)
    expect(game.state().time).toBeGreaterThan(0)
    expect(game.node('arena').width).toBe(844 * 2)
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
