import test from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { accents, paletteFor, validAccent, buttonInk, accentInk } from '../src/lib/accentPalette.ts'

function luminance(hex: string) {
  const c=[1,3,5].map(i=>parseInt(hex.slice(i,i+2),16)/255).map(v=>v<=.04045?v/12.92:((v+.055)/1.055)**2.4)
  return c[0]*.2126+c[1]*.7152+c[2]*.0722
}
function contrast(a:string,b:string){const l=[luminance(a),luminance(b)].sort((a,b)=>b-a);return (l[0]+.05)/(l[1]+.05)}
test('the web presets match the desktop presets except for the green palette',()=>{
  const source=readFileSync(new URL('../../../../client/xiaoban-desktop/src/ui/appearance.ts',import.meta.url),'utf8')
  const web=readFileSync(new URL('../src/lib/accentPalette.ts',import.meta.url),'utf8')
  const sharedPalette = (text:string) => text
    .slice(text.indexOf('export const accents:'),text.indexOf('export const DEFAULT_ACCENT'))
    .replace(/(id: 'green', name: '薄荷绿',)\n    light: [^\n]+\n    dark: [^\n]+/, '$1')
  assert.equal(sharedPalette(web),sharedPalette(source))
  assert.equal(paletteFor('green').btn,'#07C160')
  assert.equal(paletteFor('green',true).btn,'#07C160')
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
