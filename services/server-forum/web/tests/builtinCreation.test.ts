import { test } from 'node:test'
import { strict as assert } from 'node:assert'
import { readFile, access } from 'node:fs/promises'
import { BUILTIN_GAMES, builtinGame, MOBILE_ONLY_APPS, WANCISOU_ICON } from '../src/lib/builtinCreation.ts'
const root = new URL('../../../../', import.meta.url)
const publicDir = new URL('../public/builtin-games/', import.meta.url)
const mobile = new URL('client/xiaoban-multiplatform/', root)
const mobilePacks = new URL('androidApp/src/main/assets/game-packs-v2/', mobile)
const asset = (url: string) => new URL(url.split('/builtin-games/')[1], publicDir)

test('playable games match mobile order and use the manifest icons', async () => {
  const mobile = await readFile(new URL('client/xiaoban-multiplatform/common/community-core/src/commonMain/kotlin/xyz/xiangban/xiaoban/common/community/GameLibrary.kt',root),'utf8')
  const list = mobile.match(/val ids = listOf\(([^\n]+)\)/)?.[1]
  assert.ok(list)
  const androidGames = Array.from(list.matchAll(/"([^"]+)"/g), m => m[1])
  assert.deepEqual(BUILTIN_GAMES.map(game => game.id), androidGames)
  for (const game of BUILTIN_GAMES) {
    const dir = new URL(`${game.id}/`,publicDir)
    const manifest = JSON.parse(await readFile(new URL('manifest.json',dir),'utf8'))
    assert.equal(manifest.id,game.id); assert.ok(manifest.runtimeProtocol === 1 || manifest.runtimeProtocol === 2); assert.equal(manifest.entry,'index.html')
    assert.equal(game.icon,`https://res.xiangban.xyz/builtin-games/${game.id}/${manifest.icon}`)
    await access(asset(game.icon))
    const html = await readFile(new URL('index.html',dir),'utf8')
    assert.match(html,/Content-Security-Policy/); assert.match(html,/connect-src 'none'/)
  }
  assert.equal(builtinGame('../game'),undefined)
})

test('the web plaza reuses mobile tool artwork', async () => {
  for (const app of Object.values(MOBILE_ONLY_APPS)) {
    const dir = new URL(app.id + '/', mobilePacks)
    const manifest = JSON.parse(await readFile(new URL('manifest.json', dir), 'utf8'))
    assert.deepEqual(await readFile(asset(app.icon)), await readFile(new URL(manifest.icon, dir)))
  }
  assert.deepEqual(
    await readFile(asset(WANCISOU_ICON)),
    await readFile(new URL('business/community-native/src/commonMain/composeResources/files/wancisou-icon.webp', mobile)),
  )
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
