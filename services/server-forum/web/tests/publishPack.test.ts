import { strict as assert } from 'node:assert'
import { test } from 'node:test'
import JSZip from 'jszip'
import { convertPublishHtml, inspectPackZip, type PreviousPack } from '../src/lib/publishPack.ts'
import { appRuntimeEntry } from '../src/lib/appRuntime.ts'

const manifest = {
  schemaVersion: 3, runtimeProtocol: 2, kind: 'game', id: 'demo-game', name: '社区游戏',
  description: '支持鼠标与触屏的拼图', version: '1.0.0', category: 'puzzle',
  entry: 'play/main.HTM', cover: 'images/cover.webp', icon: 'images/icon.svg', input: ['touch', 'mouse'],
}
const previous: PreviousPack = { kind: 'game', slug: manifest.id, title: manifest.name, version: 1, versions: [{ manifest }] }

async function archive(overrides: Record<string, unknown> = {}, files?: (zip: JSZip) => void) {
  const zip = new JSZip()
  zip.file('manifest.json', JSON.stringify({ ...manifest, ...overrides }))
  zip.file('play/main.HTM', '<!doctype html><h1>社区游戏</h1>')
  zip.file('images/icon.svg', '<svg/>')
  zip.file('images/cover.webp', new Uint8Array([1, 2, 3]))
  files?.(zip)
  return new File([await zip.generateAsync({ type: 'blob' })], 'game.zip')
}

test('single HTML becomes a portable V2 ZIP while preserving the page and document prolog', async () => {
  const prolog = '\uFEFF<!-- 保留注释 --><!DOCTYPE html><HTML lang="zh-CN"><HEAD data-name="a>b">'
  const remainder = '<script>window.original = true</script></HEAD><body>原始内容</body></HTML>'
  const result = await convertPublishHtml(new File([prolog + remainder], '计算器<&>.HTM'))
  const zip = await JSZip.loadAsync(await result.file.arrayBuffer())
  const packed = JSON.parse(await zip.file('manifest.json')!.async('string'))
  assert.equal(packed.schemaVersion, 3)
  assert.equal(packed.runtimeProtocol, 2)
  assert.equal(packed.kind, 'app')
  assert.equal(packed.category, 'tool')
  assert.match(packed.id, /^html-[a-f0-9]{32}$/)
  assert.deepEqual(packed.capabilities, [])
  const page = await zip.file(packed.entry)!.async('string')
  assert.ok(page.startsWith(prolog))
  assert.ok(page.endsWith(remainder))
  assert.match(page.slice(prolog.length, -remainder.length), /window\.xb\?\.ready\?\.\(\)/)
  assert.match(await zip.file(packed.icon)!.async('string'), /width="192" height="192"/)
  assert.match(await zip.file(packed.cover)!.async('string'), /width="640" height="360"[\s\S]*计算器&lt;&amp;&gt;/)
  assert.equal((await inspectPackZip(result.file)).name, '计算器<&>')
})

test('HTML cannot be converted from a blank or non UTF-8 file', async () => {
  await assert.rejects(convertPublishHtml(new File(['\uFEFF \r\n'], 'empty.html')), /非空/)
  await assert.rejects(convertPublishHtml(new File([new Uint8Array([0xc0, 0xaf])], 'wrong.html')), /UTF-8/)
})

test('a V2 ZIP keeps its original bytes and the game type in its manifest', async () => {
  const file = await archive({}, zip => zip.file('assets/mesh.glb', new Uint8Array([1, 2, 3])))
  const ready = await inspectPackZip(file)
  assert.equal(ready.file, file)
  assert.equal(ready.kind, 'game')
  assert.equal(ready.protocol, 2)
  assert.equal(ready.name, manifest.name)
  assert.equal(appRuntimeEntry(new URL('https://app.xiangban.xyz'), 1, 1, manifest).pathname, '/v1/play/main.HTM')
})

test('incomplete V2 packages and old game packages are rejected before uploading a new work', async () => {
  await assert.rejects(inspectPackZip(await archive({ cover: 'missing.webp' })), /cover 文件/)
  await assert.rejects(inspectPackZip(await archive({ schemaVersion: 1, runtimeProtocol: 1 })), /V2 规范/)
  await assert.rejects(inspectPackZip(await archive({}, zip => zip.file('Images/extra.png', 'png'))), /路径冲突/)
  await assert.rejects(inspectPackZip(await archive({ description: 'x'.repeat(65 * 1024) })), /64 KB/)
})

test('an updated HTML work keeps the existing pack identity; a ZIP with another ID is refused', async () => {
  const updated = await convertPublishHtml(new File(['<!doctype html><h1>新页面</h1>'], 'new.html'), previous)
  const zip = await JSZip.loadAsync(await updated.file.arrayBuffer())
  const packed = JSON.parse(await zip.file('manifest.json')!.async('string'))
  assert.equal(packed.id, manifest.id)
  assert.equal(packed.kind, 'game')
  assert.equal(packed.category, 'puzzle')
  assert.equal(packed.version, '2.0.0')
  await inspectPackZip(updated.file, previous)
  await assert.rejects(inspectPackZip(await archive({ id: 'another-game' }), previous), /作品 ID/)
})

test('existing non-runtime content pack formats still use their declared type', async () => {
  const zip = new JSZip()
  zip.file('manifest.json', JSON.stringify({ schemaVersion: 1, kind: 'effect', id: 'glow', name: '光晕', version: '1.0.0' }))
  zip.file('payload.json', '{}')
  const file = new File([await zip.generateAsync({ type: 'blob' })], 'effect.zip')
  assert.equal((await inspectPackZip(file)).kind, 'effect')
})

test('desktop export ZIPs keep their original bytes and read names from the manifest or JSON entry', async () => {
  const examples = [
    { kind: 'wallpaper', entry: 'scene.html', name: '动态壁纸' },
    { kind: 'widget', entry: 'index.htm', id: 'tiny-clock', name: '小组件' },
    { kind: 'effect', entry: 'theme/effect.json', name: '星空特效', json: true, id: 'starlight' },
    { kind: 'pet', entry: 'pet.json', name: '小伙伴', json: true, id: 'little-fox' },
  ]
  for (const { kind, entry, id, name, json } of examples) {
    const zip = new JSZip()
    zip.file('manifest.json', JSON.stringify({ schemaVersion: 3, kind, entry, ...(json ? {} : { id, name }) }))
    zip.file(entry, json ? JSON.stringify({ schemaVersion: 1, id, name, description: '来自桌面导出' }) : '<h1>测试</h1>')
    if (kind === 'pet') zip.file('assets/Face.webp', 'RIFF....WEBP')
    const file = new File([await zip.generateAsync({ type: 'blob' })], `${kind}.zip`)
    const inspected = await inspectPackZip(file)
    assert.equal(inspected.file, file)
    assert.equal(inspected.kind, kind)
    assert.equal(inspected.name, name)
    assert.equal(inspected.desktopResource, true)
    assert.equal(inspected.protocol, 1)
    if (kind === 'wallpaper') {
      await inspectPackZip(file, { kind, slug: 'wallpaper-forum-id', title: name, version: 1, versions: [] })
    } else {
      await assert.rejects(inspectPackZip(file, { kind, slug: 'different-id', title: name, version: 1, versions: [] }), /作品 ID/)
    }
  }
})

test('a desktop ZIP needs its declared entry and keeps executable files out of effect packs', async () => {
  const zip = new JSZip()
  zip.file('manifest.json', JSON.stringify({ schemaVersion: 3, kind: 'effect', entry: 'effect.json' }))
  const file = async () => new File([await zip.generateAsync({ type: 'blob' })], 'effect.zip')
  await assert.rejects(inspectPackZip(await file()), /缺少入口/)
  zip.file('effect.json', JSON.stringify({ schemaVersion: 1, id: 'starlight', name: '星空特效' }))
  zip.file('extra.html', '<h1>额外文件</h1>')
  await assert.rejects(inspectPackZip(await file()), /额外文件/)
})
