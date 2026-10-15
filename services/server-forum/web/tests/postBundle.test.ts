import assert from 'node:assert/strict'
import { test } from 'node:test'
import JSZip from 'jszip'
import { parsePost, readBundle } from '../src/lib/postBundle.ts'
const text = '---\nid: demo-001\ntitle: 一篇具体的教程\ncategory: tech\ntags: [教程]\n---\n\n正文包含具体的方法和操作步骤。\n![图](images/图.png)'
test('ZIP supports a top folder, Unicode images and per-post errors', async () => {
  const zip = new JSZip(); zip.file('posts/a.md', text); zip.file('posts/images/图.png', new Uint8Array([137, 80, 78, 71])); zip.file('posts/b.md', 'missing metadata')
  const rows = await readBundle(new File([await zip.generateAsync({ type: 'uint8array' })], 'batch.zip'))
  assert.equal(rows[0]!.data!.images[0]!.path, 'images/图.png'); assert.equal(rows[1]!.selected, false); assert.ok(rows[1]!.error)
})
test('rejects duplicate metadata and duplicate ids', async () => {
  assert.throws(() => parsePost(text.replace('id: demo-001', 'id: a\nid: b')))
  const zip = new JSZip(); zip.file('a.md', text); zip.file('b.md', text)
  const rows = await readBundle(new File([await zip.generateAsync({ type: 'uint8array' })], 'batch.zip'))
  assert.match(rows[1]!.error!, /重复/)
})
test('rejects normalized ZIP traversal before reading contents', async () => {
  const zip = new JSZip(); zip.file('../escape.md', text)
  await assert.rejects(readBundle(new File([await zip.generateAsync({ type: 'uint8array' })], 'bad.zip')), /路径/)
})
