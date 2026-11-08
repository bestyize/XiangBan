import { test } from 'node:test'
import { strict as assert } from 'node:assert'
import { readFile, access } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import { BUILTIN_GAMES, builtinGame } from '../src/lib/builtinCreation.ts'
const root = new URL('../../../../', import.meta.url)
const publicDir = new URL('../public/builtin-games/', import.meta.url)
test('built-in games match mobile order and copied official manifests', async () => {
  const mobile = await readFile(new URL('client/xiaoban-multiplatform/common/community-core/src/commonMain/kotlin/xyz/xiangban/xiaoban/common/community/GameLibrary.kt',root),'utf8')
  const list = mobile.match(/val ids = listOf\(([^\n]+)\)/)?.[1]
  assert.ok(list)
  assert.deepEqual(BUILTIN_GAMES.map(game => game.id), Array.from(list.matchAll(/"([^"]+)"/g), m => m[1]))
  for (const game of BUILTIN_GAMES) {
    const dir = new URL(`${game.id}/`,publicDir)
    const manifest = JSON.parse(await readFile(new URL('manifest.json',dir),'utf8'))
    assert.equal(manifest.id,game.id); assert.equal(manifest.name,game.name); assert.ok(manifest.runtimeProtocol === 1 || manifest.runtimeProtocol === 2); assert.equal(manifest.entry,'index.html')
    assert.equal(game.cover,`/builtin-games/${game.id}/cover.svg`); await access(fileURLToPath(new URL('cover.svg',dir)))
    const html = await readFile(new URL('index.html',dir),'utf8')
    assert.match(html,/Content-Security-Policy/); assert.match(html,/connect-src 'none'/)
  }
  assert.equal(builtinGame('../game'),undefined)
})

test('game handshake accepts only the current opaque frame and nonce', async () => {
  const { isBuiltinGameMessage } = await import('../src/lib/builtinCreation.ts')
  const frame = {}, other = {}, message = { channel:'xiaban.game-pack',protocol:1,type:'game.ready',nonce:'one' }
  assert.equal(isBuiltinGameMessage(message,'null',frame,frame,'one'),true)
  assert.equal(isBuiltinGameMessage(message,'null',other,frame,'one'),false)
  assert.equal(isBuiltinGameMessage(message,'https://forum.example',frame,frame,'one'),false)
  assert.equal(isBuiltinGameMessage(message,'null',frame,frame,'two'),false)
  assert.equal(isBuiltinGameMessage(message,'null',null,undefined,'one'),false)
})
