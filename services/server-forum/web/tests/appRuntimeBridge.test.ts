import { test } from 'node:test'
import { strict as assert } from 'node:assert'
import { readFile } from 'node:fs/promises'
import vm from 'node:vm'

const source = await readFile(new URL('../runtime-v2/bridge.js', import.meta.url), 'utf8')
const channel = 'xiaoban.app-runtime.v2'
const parentOrigin = 'https://chat.xiangban.xyz'

function load(options: { framed?: boolean } = {}) {
  const framed = options.framed !== false
  const listeners = new Map<string, Array<(event: any) => void>>()
  const posted: Array<{ message: any; origin: string }> = []
  const parent = { postMessage: (message: any, origin: string) => posted.push({ message, origin }) }
  const media = { matches: false, addEventListener() {} }
  const window: any = {
    addEventListener: (name: string, fn: (event: any) => void) => listeners.set(name, [...(listeners.get(name) ?? []), fn]),
  }
  window.parent = framed ? parent : window
  const document = {
    hidden: false,
    currentScript: { dataset: { parentOrigin } },
    documentElement: { style: { setProperty() {} } },
    addEventListener: (name: string, fn: (event: any) => void) => listeners.set(`doc:${name}`, [...(listeners.get(`doc:${name}`) ?? []), fn]),
  }
  const context: any = {
    window, document, parent: window.parent, navigator: { language: 'zh-CN' }, matchMedia: () => media,
    addEventListener: window.addEventListener, setTimeout, clearTimeout, console, Object, Promise, Number, Map, Set, Error,
  }
  Object.defineProperty(context, 'window', { value: window })
  window.window = window
  context.self = window
  vm.createContext(context)
  vm.runInContext(source, context)
  const xb = Object.getOwnPropertyDescriptor(window, 'xb')?.value ?? context.xb
  const deliver = (data: unknown, extra: Record<string, unknown> = {}) => {
    for (const fn of listeners.get('message') ?? []) fn({ source: parent, origin: parentOrigin, data, ...extra })
  }
  return { xb, posted, parent, deliver, listeners, document }
}

test('the bridge exposes a frozen xb with no optional capabilities', async () => {
  const { xb } = load()
  assert.equal(xb.info.platform, 'web')
  assert.equal(xb.can('camera'), false)
  assert.ok(Object.isFrozen(xb) && Object.isFrozen(xb.storage) && Object.isFrozen(xb.game))
  assert.equal(xb.info.legacySave, null)
})

test('ready and exit notify only the configured parent origin', async () => {
  const { xb, posted } = load()
  await xb.ready(); await xb.ready(); await xb.exit()
  assert.deepEqual(posted.map(item => item.message.type), ['ready', 'exit'])
  assert.ok(posted.every(item => item.origin === parentOrigin && item.message.channel === channel))
})

test('storage requests carry an id and resolve from a matching reply', async () => {
  const { xb, posted, deliver } = load()
  const pending = xb.storage.set('save', { best: 3 })
  const request = posted[0].message
  assert.deepEqual({ type: request.type, operation: request.operation, key: request.key, value: request.value }, { type: 'storage', operation: 'set', key: 'save', value: { best: 3 } })
  deliver({ channel, type: 'storage-result', id: request.id, value: null })
  assert.equal(await pending, null)
  const read = xb.storage.get('save')
  deliver({ channel, type: 'storage-result', id: posted[1].message.id, value: { best: 3 } })
  assert.deepEqual(await read, { best: 3 })
})

test('storage errors surface their code and replies for unknown ids are ignored', async () => {
  const { xb, posted, deliver } = load()
  const pending = xb.storage.set('big', 'x')
  deliver({ channel, type: 'storage-result', id: 999, value: 'stray' })
  deliver({ channel, type: 'storage-result', id: posted[0].message.id, error: 'quota' })
  await assert.rejects(pending, (error: any) => error.code === 'quota')
})

test('messages from another window, origin or channel are ignored', async () => {
  const { xb, posted, deliver, parent } = load()
  const pending = xb.storage.get('save')
  const id = posted[0].message.id
  deliver({ channel, type: 'storage-result', id, value: 'spoof' }, { origin: 'https://evil.example' })
  deliver({ channel, type: 'storage-result', id, value: 'spoof' }, { source: {} })
  deliver({ channel: 'other', type: 'storage-result', id, value: 'spoof' })
  deliver({ channel, type: 'storage-result', id, value: 'real' })
  assert.equal(await pending, 'real')
  assert.ok(parent)
})

test('storage outside a host frame is unsupported', async () => {
  const { xb } = load({ framed: false })
  await assert.rejects(xb.storage.get('save'), (error: any) => error.code === 'unsupported')
})

test('scores need a started round and a non-negative integer', async () => {
  const { xb } = load()
  await assert.rejects(xb.game.progress({ score: 1 }), (error: any) => error.code === 'invalid')
  await xb.game.start()
  await xb.game.progress({ score: 0 })
  for (const score of [-1, 1.5, Number.NaN, '3' as unknown as number]) await assert.rejects(xb.game.progress({ score }), undefined, String(score))
  await xb.game.result({ score: 7 })
  await assert.rejects(xb.game.result({ score: 7 }), (error: any) => error.code === 'invalid')
})

test('hide and show events follow the host and visibility changes', async () => {
  const { xb, deliver } = load()
  const seen: string[] = []
  xb.on('hide', () => seen.push('hide')); xb.on('show', () => seen.push('show'))
  deliver({ channel, type: 'hide' })
  deliver({ channel, type: 'hide' })
  deliver({ channel, type: 'show' })
  assert.deepEqual(seen, ['hide', 'show'])
})
