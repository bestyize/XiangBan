import { test } from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { BUILTIN_WIDGETS } from '../src/lib/builtinWidgets.ts'

test('web catalog matches desktop widgetKinds and bundled custom widget', () => {
  const model = readFileSync(new URL('../../../../client/xiaoban-desktop/src/widgets/model.ts', import.meta.url), 'utf8')
  const block = model.match(/export const widgetKinds = \[([\s\S]*?)\];/)?.[1] ?? ''
  const desktop = [...block.matchAll(/kind:'([^']+)'\s+as const,title:'([^']+)',label:'[^']+',description:'([^']+)'/g)].map(([, id, name, description]) => ({ id, name, description, kind:'widget' }))
  const custom = JSON.parse(readFileSync(new URL('../../../../client/xiaoban-desktop/public/widgets/pelican-bike.json', import.meta.url), 'utf8'))
  desktop.push({ id:custom.id, name:custom.name, description:custom.description, kind:'widget' })
  assert.equal(desktop.length, 9)
  assert.deepEqual(BUILTIN_WIDGETS, desktop)
})
