import { test } from 'node:test'
import { strictEqual } from 'node:assert'
import { areaOf, safeReturnPath, sectionOf } from '../src/lib/sections.ts'

test('login destinations keep internal query/hash and reject external or control-character URLs', () => {
  strictEqual(safeReturnPath('/publish?pack=55#details'), true)
  for (const path of ['https://example.com', '//example.com', '/\\example.com', '/\r\nexample.com']) {
    strictEqual(safeReturnPath(path), false)
  }
})

test('details and legacy scheme pages retain their parent navigation', () => {
  for (const name of ['pack', 'schemes', 'scheme', 'scheme-publish']) strictEqual(areaOf({ name, params: {} }), 'create')
  strictEqual(sectionOf({name:'tool-detail',params:{key:'example'}}), 'resources')
  strictEqual(sectionOf({name:'daily-detail',params:{key:'example'}}), 'home')
  strictEqual(areaOf({name:'content-bookmarks',params:{}}), 'me')
})
