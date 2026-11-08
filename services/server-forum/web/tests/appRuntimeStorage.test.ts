import { test } from 'node:test'
import { strict as assert } from 'node:assert'

// A minimal in-memory IndexedDB: only the calls appRuntimeStorage makes.
type Key = [number, string]
const cmp = (a: Key, b: Key) => a[0] - b[0] || (a[1] < b[1] ? -1 : a[1] > b[1] ? 1 : 0)
const rows = new Map<string, { key: Key; value: string }>()
const id = (key: Key) => `${key[0]}\u0000${key[1]}`
const later = (fn: () => void) => queueMicrotask(fn)
;(globalThis as any).IDBKeyRange = { bound: (lower: Key, upper: Key) => ({ lower, upper }) }
const inRange = (key: Key, range: { lower: Key; upper: Key }) => cmp(key, range.lower) >= 0 && cmp(key, range.upper) < 0
;(globalThis as any).indexedDB = {
  open() {
    const request: any = {}
    const database = {
      createObjectStore() {},
      transaction() {
        const tx: any = {}
        let finished = false
        const done = () => { if (!finished) { finished = true; tx.oncomplete?.() } }
        tx.abort = () => { finished = true; later(() => tx.onabort?.()) }
        tx.objectStore = () => ({
          get(key: Key) { const r: any = {}; later(() => { r.result = rows.get(id(key))?.value; r.onsuccess?.(); later(done) }); return r },
          delete(key: Key) { rows.delete(id(key)); later(done) },
          put(value: string, key: Key) { rows.set(id(key), { key, value }); later(done) },
          getAllKeys(range: any) {
            const r: any = {}
            later(() => { r.result = [...rows.values()].filter(row => inRange(row.key, range)).map(row => row.key); r.onsuccess?.(); later(done) })
            return r
          },
          openCursor(range: any) {
            const r: any = {}
            const list = [...rows.values()].filter(row => inRange(row.key, range))
            let index = 0
            const step = () => {
              const row = list[index]
              r.result = row ? { key: row.key, value: row.value, continue: () => { index++; later(step) } } : null
              r.onsuccess?.()
              if (!row) later(done)
            }
            later(step)
            return r
          },
        })
        return tx
      },
    }
    later(() => { request.result = database; request.onupgradeneeded?.(); request.onsuccess?.() })
    return request
  },
}
const { appRuntimeStorage } = await import('../src/lib/appRuntimeStorage.ts')

test('values round trip as JSON and a missing key is null', async () => {
  assert.equal(await appRuntimeStorage(1, 'get', 'save'), null)
  await appRuntimeStorage(1, 'set', 'save', { best: 3, list: [1, 'a', null] })
  assert.deepEqual(await appRuntimeStorage(1, 'get', 'save'), { best: 3, list: [1, 'a', null] })
  await appRuntimeStorage(1, 'set', 'save', 'replaced')
  assert.equal(await appRuntimeStorage(1, 'get', 'save'), 'replaced')
})

test('each pack only sees its own keys', async () => {
  await appRuntimeStorage(10, 'set', 'a', 1)
  await appRuntimeStorage(11, 'set', 'a', 2)
  await appRuntimeStorage(11, 'set', 'b', 3)
  assert.equal(await appRuntimeStorage(10, 'get', 'a'), 1)
  assert.equal(await appRuntimeStorage(11, 'get', 'a'), 2)
  assert.deepEqual(await appRuntimeStorage(10, 'keys'), ['a'])
  assert.deepEqual((await appRuntimeStorage(11, 'keys') as string[]).sort(), ['a', 'b'])
  await appRuntimeStorage(11, 'remove', 'a')
  assert.equal(await appRuntimeStorage(11, 'get', 'a'), null)
  assert.equal(await appRuntimeStorage(10, 'get', 'a'), 1)
})

test('invalid operations, keys and values are refused', async () => {
  await assert.rejects(appRuntimeStorage(2, 'drop', 'k'), /invalid/)
  await assert.rejects(appRuntimeStorage(2, 'get', ''), /invalid/)
  await assert.rejects(appRuntimeStorage(2, 'get', 5), /invalid/)
  await assert.rejects(appRuntimeStorage(2, 'get', 'k'.repeat(129)), /invalid/)
  await assert.rejects(appRuntimeStorage(2, 'set', 'k', undefined), /invalid/)
  await assert.rejects(appRuntimeStorage(2, 'set', 'k', () => 1), /invalid/)
  const cycle: any = {}; cycle.self = cycle
  await assert.rejects(appRuntimeStorage(2, 'set', 'k', cycle), /invalid/)
})

test('one pack cannot store more than ten megabytes in total', async () => {
  const chunk = 'x'.repeat(4 * 1024 * 1024)
  await appRuntimeStorage(3, 'set', 'one', chunk)
  await appRuntimeStorage(3, 'set', 'two', chunk)
  await assert.rejects(appRuntimeStorage(3, 'set', 'three', chunk), /quota/)
  await assert.rejects(appRuntimeStorage(3, 'set', 'huge', 'x'.repeat(11 * 1024 * 1024)), /quota/)
  await appRuntimeStorage(3, 'set', 'one', 'small')
  await appRuntimeStorage(3, 'set', 'three', chunk)
  assert.equal(await appRuntimeStorage(3, 'get', 'three'), chunk)
  assert.equal(await appRuntimeStorage(4, 'get', 'one'), null)
})
