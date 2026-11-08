import { test } from 'node:test'
import { strict as assert } from 'node:assert'
import { appRuntimeBase, appRuntimeEntry, canTrialPack } from '../src/lib/appRuntime.ts'

const manifest = { schemaVersion: 3, runtimeProtocol: 2 }

test('published protocol 2 community works remain playable at trust level 2', () => {
  assert.equal(canTrialPack('game', 'published', 2, 2), true)
  assert.equal(canTrialPack('app', 'published', 2, 2), true)
  assert.equal(canTrialPack('game', 'published', 2, 1), false)
  assert.equal(canTrialPack('game', 'disabled', 0, 2), false)
})

test('only an https origin without credentials, port or path is a runtime base', () => {
  assert.equal(appRuntimeBase('https://app.xiangban.xyz', false)?.hostname, 'app.xiangban.xyz')
  for (const bad of [undefined, '', 'http://app.xiangban.xyz', 'https://app.xiangban.xyz:8443', 'https://u:p@app.xiangban.xyz',
    'https://app.xiangban.xyz/path', 'https://app.xiangban.xyz/?q=1', 'https://app.xiangban.xyz/#x', 'https://localhost', 'not a url']) {
    assert.equal(appRuntimeBase(bad, false), null, String(bad))
  }
  assert.equal(appRuntimeBase('http://app.localhost:5201', false), null)
  assert.equal(appRuntimeBase('http://app.localhost:5201', true)?.hostname, 'app.localhost')
  assert.equal(appRuntimeBase('http://evil.example:5201', true), null)
})

test('each pack version gets its own origin and versioned path', () => {
  const base = new URL('https://app.xiangban.xyz')
  assert.equal(appRuntimeEntry(base, 56, 3, manifest).href, 'https://56.app.xiangban.xyz/v3/index.html')
  assert.equal(appRuntimeEntry(base, 57, 1, { ...manifest, entry: 'play/main.html' }).href, 'https://57.app.xiangban.xyz/v1/play/main.html')
  assert.notEqual(appRuntimeEntry(base, 56, 1, manifest).origin, appRuntimeEntry(base, 57, 1, manifest).origin)
})

test('invalid ids, versions, protocols and entries are refused', () => {
  const base = new URL('https://app.xiangban.xyz')
  for (const [id, version] of [[0, 1], [-1, 1], [1.5, 1], [1, 0], [1, 2.5], [Number.NaN, 1]]) {
    assert.throws(() => appRuntimeEntry(base, id, version, manifest), undefined, `${id}/${version}`)
  }
  assert.throws(() => appRuntimeEntry(base, 1, 1, { schemaVersion: 1, runtimeProtocol: 1 }))
  assert.throws(() => appRuntimeEntry(base, 1, 1, { schemaVersion: 3, runtimeProtocol: '2' }))
  for (const entry of ['../index.html', '/index.html', 'a//b.html', '.hidden.html', 'index.js', 'a\\b.html', 1, '']) {
    assert.throws(() => appRuntimeEntry(base, 1, 1, { ...manifest, entry }), undefined, String(entry))
  }
})
