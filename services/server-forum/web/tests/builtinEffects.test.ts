import { test } from 'node:test'
import { strict as assert } from 'node:assert'
import { readFile } from 'node:fs/promises'
const desktop = new URL('../../../../client/xiaoban-desktop/public/effects/', import.meta.url)
const local = new URL('../src/lib/builtinEffectsData/', import.meta.url)
test('web effects are the twelve official desktop packs in catalog order', async () => {
  const list = JSON.parse(await readFile(new URL('catalog.json',desktop),'utf8')).effects as string[]
  assert.equal(list.length,12); assert.equal(new Set(list).size,12)
  const module = await readFile(new URL('../src/lib/builtinEffects.ts',import.meta.url),'utf8')
  const bindingIds = Array.from(module.matchAll(/import \w+ from '\.\/builtinEffectsData\/([^']+)\.json'/g), m => m[1])
  assert.deepEqual(bindingIds,list)
  for (const id of list) {
    const source = await readFile(new URL(`${id}.json`,desktop),'utf8'), copy = await readFile(new URL(`${id}.json`,local),'utf8')
    assert.equal(copy,source); const pack=JSON.parse(copy); assert.equal(pack.id,id); assert.ok(pack.name && pack.description && pack.layers.length)
  }
})


test('animation pauses for offscreen, background and reduced motion', async () => {
  const { shouldAnimateEffect } = await import('../src/lib/builtinEffectsPlayback.ts')
  assert.equal(shouldAnimateEffect(true,true,false,false),true)
  assert.equal(shouldAnimateEffect(false,true,false,false),false)
  assert.equal(shouldAnimateEffect(true,false,false,false),false)
  assert.equal(shouldAnimateEffect(true,true,true,false),false)
  assert.equal(shouldAnimateEffect(true,true,false,true),false)
})
