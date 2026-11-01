import assert from 'node:assert/strict'
import test from 'node:test'
import { createRequestGate, discoverPaths, officialSource, repoGrowth, repoMetric, repoPage, webLink } from '../src/components/community/contract.ts'

test('repo detail paths escape each segment and only valid GitHub names get an in-site page', () => {
  assert.equal(discoverPaths.repo('own er', 'a/b'), '/discover/repo/own%20er/a%2Fb')
  assert.equal(discoverPaths.repoComments('o', 'r'), '/discover/repo/o/r/comments')
  assert.equal(repoPage('Nutlope/hallmark'), '/repo/Nutlope/hallmark')
  assert.equal(repoPage('a-b/c.d_e'), '/repo/a-b/c.d_e')
  for (const bad of ['', 'owner', 'a/b/c', '/b', 'a/', 'a b/c', 'a/../b', 'a/b?x=1', 'a/b#h', 'a%2Fb/c', 'x'.repeat(40) + '/r']) assert.equal(repoPage(bad), undefined, bad)
})

test('discover paths match the versioned mobile contract and escape keys once', () => {
  assert.equal(discoverPaths.home, '/discover/home/v2')
  assert.equal(discoverPaths.repos, '/discover/repos/v2')
  assert.equal(discoverPaths.issues, '/discover/repos/issues/v2')
  assert.equal(discoverPaths.tracker, '/discover/tracker/v2')
  assert.equal(discoverPaths.article('a/b?c#d'), '/discover/daily/a%2Fb%3Fc%23d')
  assert.equal(discoverPaths.comments('a/b'), '/discover/daily/a%2Fb/comments')
  assert.equal(discoverPaths.tool('tool#1'), '/tools/tool%231')
})

test('editorial links reject scripts, credentials, cross-origin path tricks and controls', () => {
  for (const raw of ['javascript:alert(1)', 'data:image/svg+xml,a', '//evil.test', '/\\evil.test', 'https://user:pass@example.com', 'https://example.com\n/path', 'file:///etc/passwd']) assert.equal(webLink(raw), undefined, raw)
  assert.equal(webLink('/api/images/1'), '/api/images/1')
  assert.equal(webLink('https://github.com/example/project'), 'https://github.com/example/project')
  assert.equal(officialSource('http://example.com'), undefined)
  assert.equal(officialSource('https://example.com:8443'), undefined)
  assert.equal(officialSource('/api/tool'), undefined)
  assert.equal(officialSource('https://example.com'), 'https://example.com/')
})

test('baseline and incomplete observations never invent weekly growth', () => {
  assert.equal(repoMetric({ source:'GitHub',metric:'star total baseline',complete:false }), 'Star 总量基线')
  assert.equal(repoMetric({ source:'GitHub',metric:'star gain',complete:false }), '观察期 Star 增量')
  assert.equal(repoMetric({ source:'GitHub',metric:'star gain',complete:true }), '周期 Star 增量')
  assert.equal(repoGrowth(null), '尚无可比增量')
  assert.equal(repoGrowth(undefined), '尚无可比增量')
  assert.equal(repoGrowth(0), '+0')
  assert.equal(repoGrowth(-12), '-12')
})

test('a slower old request and disposed requests cannot replace current content', async () => {
  const gate = createRequestGate()
  let resolveOld!: (value:string) => void
  const pending = new Promise<string>(resolve => { resolveOld = resolve })
  const old = gate.next()
  let value = ''
  const oldWork = pending.then(result => { if (gate.current(old)) value = result })
  const latest = gate.next()
  if (gate.current(latest)) value = 'current filter'
  resolveOld('old filter')
  await oldWork
  assert.equal(value, 'current filter')
  gate.cancel()
  assert.equal(gate.current(latest), false)
  assert.equal(gate.current(gate.next()), true)
})
