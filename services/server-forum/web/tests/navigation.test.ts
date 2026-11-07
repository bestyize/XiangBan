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
  for (const name of ['open-source','open-source-project','open-source-collection']) strictEqual(sectionOf({name,params:{}}), 'home')
  strictEqual(areaOf({name:'content-bookmarks',params:{}}), 'me')
})

test('canonical resource category URLs belong to the resource section',()=>{
 for(const slug of ['resources','resource-movies','resource-help']) strictEqual(sectionOf({name:'category',params:{slug}},[{slug,section:'resources'}]),'resources')
})

test('custom category slugs follow metadata instead of historical prefixes',()=>{
 const categories=[{slug:'my-tools',section:'resources' as const},{slug:'create-example',section:'discuss' as const},{slug:'custom-art',section:'ideas' as const}]
 strictEqual(sectionOf({name:'category',params:{slug:'my-tools'}},categories),'resources')
 strictEqual(sectionOf({name:'category',params:{slug:'create-example'}},categories),'discuss')
 strictEqual(sectionOf({name:'category',params:{slug:'custom-art'}},categories),'ideas')
})
