import test from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { accents, paletteFor, validAccent, buttonInk, accentInk } from '../src/lib/accentPalette.ts'

function luminance(hex: string) {
  const c=[1,3,5].map(i=>parseInt(hex.slice(i,i+2),16)/255).map(v=>v<=.04045?v/12.92:((v+.055)/1.055)**2.4)
  return c[0]*.2126+c[1]*.7152+c[2]*.0722
}
function contrast(a:string,b:string){const l=[luminance(a),luminance(b)].sort((a,b)=>b-a);return (l[0]+.05)/(l[1]+.05)}
test('all six web palettes retain the official client names and colours',()=>{
  const source=readFileSync(new URL('../../../../client/xiaoban-desktop/src/ui/appearance.ts',import.meta.url),'utf8')
  const web=readFileSync(new URL('../src/lib/accentPalette.ts',import.meta.url),'utf8')
  const palette=(s:string)=>s.slice(s.indexOf('export const accents:'),s.indexOf('export const DEFAULT_ACCENT'))
  assert.equal(palette(web),palette(source))
  assert.equal(accents.length,6)
  assert.equal(validAccent('invalid'),false)
  assert.deepEqual(paletteFor('invalid'),accents[0].light)
})
test('accent labels and solid button text stay readable in both appearances',()=>{
  for(const option of accents) for(const dark of [false,true]) {
    const p=paletteFor(option.id,dark)
    assert.ok(contrast(buttonInk(p.btn),p.btn)>=4.5,option.id+' button')
    assert.ok(contrast(accentInk(p,dark),dark?'#50555f':'#f2f2f2')>=4.5,option.id+' text')
  }
})
