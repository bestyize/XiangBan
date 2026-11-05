import assert from 'node:assert/strict'
import test from 'node:test'
import { lookupMainlandChina, softwareDownloadUrl } from '../src/lib/softwareDownload.ts'

const official = 'https://github.com/owner/repo/releases/download/v1/tool.exe'

test('only a successful mainland country lookup selects the accelerated download', async () => {
  const lookup = (body: unknown, status = 200) => lookupMainlandChina(async () =>
    new Response(JSON.stringify(body), { status }))
  assert.equal(await lookup({ success: true, country_code: 'CN' }), true)
  for (const country_code of ['US', 'HK', 'MO', 'TW', '']) {
    assert.equal(await lookup({ success: true, country_code }), false)
  }
  assert.equal(await lookup({ success: false, country_code: 'CN' }), false)
  assert.equal(await lookup({ success: true, country_code: 'CN' }, 429), false)
  assert.equal(await lookupMainlandChina(async () => { throw new Error('network') }), false)
})

test('only GitHub release assets use the proxy for mainland visitors', () => {
  assert.equal(softwareDownloadUrl(official, true), `https://gh-proxy.com/${official}`)
  assert.equal(softwareDownloadUrl(official, false), official)
  for (const url of ['https://github.com.evil.example/a/b/releases/download/v1/a.exe',
    'https://github.com/a/b/releases/tag/v1', 'https://user@github.com/a/b/releases/download/v1/a.exe']) {
    assert.equal(softwareDownloadUrl(url, true), url)
  }
})
