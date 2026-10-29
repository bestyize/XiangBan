import test from 'node:test'
import assert from 'node:assert/strict'
import { filtersFromQuery, queryFromFilters, pluginPath, releasePath, installPrompt,
  fetchPlugins, fetchJson } from '../src/plugins/api.ts'

test('URL filters survive sharing, pagination and browser navigation', () => {
  const filters = filtersFromQuery({ q: '写作', category: 'content-creation', source: 'enterprise', sort: 'downloads', page: '3' })
  assert.deepEqual(filtersFromQuery(queryFromFilters(filters)), filters)
  assert.equal(filters.keyword, '写作')
  assert.equal(filters.category, 'content-creation')
  for (const page of ['0', '-1', '1.5', 'Infinity', '10001']) assert.equal(filtersFromQuery({ page }).page, 1)
  assert.equal(filtersFromQuery({ category: '../private', source: 'bad', sort: 'bad' }).sortBy, 'score')
  assert.equal(filtersFromQuery({ category: '../private' }).category, '')
})

test('publisher and fixed versions remain in detail/download/readme coordinates', () => {
  const entry = { slug: 'demo-skill', namespace: 'author_one', version: '1.2.3+build' }
  assert.equal(pluginPath(entry.slug, entry.namespace), '/api/v1/ai/plugins/demo-skill?namespace=author_one')
  for (const action of ['readme', 'download']) {
    const url = new URL(releasePath(entry, action), 'https://example.com')
    assert.equal(url.searchParams.get('version'), '1.2.3+build')
    assert.equal(url.searchParams.get('namespace'), 'author_one')
  }
  assert.throws(() => releasePath({ ...entry, version: null }, 'download'))
  assert.match(installPrompt(entry), /author_one\/demo-skill/)
  assert.match(installPrompt(entry), /等待我的确认/)
})

test('browsing uses only the Xiaoban API and preserves independent filters', async t => {
  const controller = new AbortController()
  t.mock.method(globalThis, 'fetch', async (input, init) => {
    const url = new URL(input, 'https://example.com')
    assert.equal(url.pathname, '/api/v1/ai/plugins')
    assert.equal(url.searchParams.get('keyword'), '写作')
    assert.equal(url.searchParams.get('category'), 'content-creation')
    assert.equal(url.searchParams.get('source'), 'enterprise')
    assert.equal(url.searchParams.get('page'), '2')
    assert.equal(init.signal, controller.signal)
    return Response.json({ results: [], page: 2, total: 0, hasMore: false })
  })
  await fetchPlugins({ keyword: '写作', category: 'content-creation', source: 'enterprise', sortBy: 'downloads', page: 2 }, controller.signal)
})

test('upstream failures are shown as errors, never as an empty successful list', async t => {
  t.mock.method(globalThis, 'fetch', async () => Response.json({ error: '请稍后重试' }, { status: 429 }))
  await assert.rejects(fetchJson('/api/v1/ai/plugins'), /请稍后重试/)
  globalThis.fetch.mock.mockImplementation(async () => new Response('Bad gateway', { status: 502 }))
  await assert.rejects(fetchJson('/api/v1/ai/plugins'), /502/)
})
