import test from 'node:test'
import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import { BUILTIN_PETS } from '../src/lib/builtinPets.ts'

test('the website lists every official desktop companion with its real identity',async () => {
  const catalog = JSON.parse(await readFile(new URL('../../../../client/xiaoban-desktop/src/pets/catalog.json', import.meta.url),'utf8'))
  const official = catalog.themes.flatMap((theme:{pets:{id:string;name:string}[]}) => theme.pets)
  assert.deepEqual(BUILTIN_PETS.map(p=>[p.id,p.name]),official.map((p:{id:string;name:string})=>[p.id,p.name]))
  assert.equal(new Set(BUILTIN_PETS.map(p=>p.id)).size,BUILTIN_PETS.length)
  assert.ok(BUILTIN_PETS.every(p=>p.kind==='pet'))
})

test('official previews contain the complete pot and face and no native or remote code',async () => {
  for (const pet of BUILTIN_PETS) {
    const svg = await readFile(new URL(`../public${pet.preview}`,import.meta.url),'utf8')
    assert.match(svg,/viewBox="0 -72 320 352"/)
    assert.match(svg,/data-part="body"/)
    assert.match(svg,/data-part="face"/)
    assert.match(svg,/M70 56A90 10/) // official porcelain pot outline
    if (pet.id !== 'green-sprout') assert.match(svg,/data:image\/webp;base64,/)
    assert.doesNotMatch(svg,/<script|foreignObject|(?:href|src)="https?:|@tauri|invoke\(/i)
  }
})
