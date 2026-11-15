import { test } from 'node:test'
import { strict as assert } from 'node:assert'
import { appRuntimeBase, appRuntimeEntry, canTrialPack } from '../src/lib/appRuntime.ts'
import { readIdeaPlayerDocument } from '../src/lib/ideaPlayerDocument.ts'

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

const ideaUrl = new URL('https://forum.example/idea-players/v1/aquarium.html')
const ideaHtml = '<!doctype html><html><head><script>\ndocument.currentScript.dataset.parentOrigin = location.origin;\n</script></head><body>水族箱</body></html>'
const ideaPolicy = "sandbox allow-scripts; frame-ancestors 'self'; default-src 'none'; script-src 'unsafe-inline'"

test('a static gzip player reports its decoded size and keeps the response policy in srcdoc', async t => {
  const size = Buffer.byteLength(ideaHtml)
  t.mock.method(globalThis, 'fetch', async () => new Response(ideaHtml, {
    headers: {
      'content-type': 'text/html',
      'content-security-policy': ideaPolicy,
      'content-encoding': 'gzip',
      'last-modified': 'Thu, 01 Jan 1970 00:00:16 GMT',
      etag: `W/"10-${size.toString(16)}"`,
      server: 'nginx',
    },
  }))
  const progress: [number, number | undefined][] = []
  const srcdoc = await readIdeaPlayerDocument(ideaUrl, 'allow-scripts', new AbortController().signal,
    (received, total) => progress.push([received, total]))
  assert.deepEqual(progress, [[0, size], [size, size]])
  assert.ok(srcdoc)
  assert.match(srcdoc, /<meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'unsafe-inline'">/)
  assert.match(srcdoc, /<script data-parent-origin="https:\/\/forum.example">/)
})

test('responses with a stricter framing or sandbox policy keep their normal navigation', async t => {
  let headers: Record<string, string> = {}
  t.mock.method(globalThis, 'fetch', async () => new Response(ideaHtml, { headers }))
  const restrictions: Record<string, string>[] = [
    { 'content-security-policy': ideaPolicy.replace("'self'", "'none'") },
    { 'content-security-policy': ideaPolicy, 'x-frame-options': 'DENY' },
    { 'content-security-policy': ideaPolicy.replace('sandbox allow-scripts', 'sandbox') },
  ]
  for (const restriction of restrictions) {
    headers = { 'content-type': 'text/html', ...restriction }
    const srcdoc = await readIdeaPlayerDocument(ideaUrl, 'allow-scripts', new AbortController().signal,
      () => assert.fail('unsupported responses should not be transferred into srcdoc'))
    assert.equal(srcdoc, undefined)
  }
})

test('a player without a total reports actual received bytes', async t => {
  t.mock.method(globalThis, 'fetch', async () => new Response(ideaHtml, {
    headers: { 'content-type': 'text/html', 'content-security-policy': ideaPolicy },
  }))
  const progress: [number, number | undefined][] = []
  await readIdeaPlayerDocument(ideaUrl, 'allow-scripts', new AbortController().signal,
    (received, total) => progress.push([received, total]))
  assert.deepEqual(progress, [[0, undefined], [Buffer.byteLength(ideaHtml), undefined]])
})
