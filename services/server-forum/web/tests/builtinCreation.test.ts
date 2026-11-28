import { test } from 'node:test'
import { strict as assert } from 'node:assert'
import { readFile, access } from 'node:fs/promises'
import { BUILTIN_GAMES, builtinGame, canOpenBundledWancisou, MOBILE_ONLY_APPS, WANCISOU_ICON } from '../src/lib/builtinCreation.ts'
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

test('wancisou V2 assets stay in sync and run without inline CSP exceptions', async () => {
  const source = new URL('assets/app-packs/wancisou/', root)
  const bundled = new URL('wancisou/', publicDir)
  const manifest = JSON.parse(await readFile(new URL('manifest.json', source), 'utf8'))
  assert.equal(manifest.version, '2.0.1')
  assert.equal(manifest.schemaVersion, 3)
  assert.equal(manifest.runtimeProtocol, 2)
  assert.deepEqual(manifest.capabilities, ['clipboard'])
  assert.deepEqual(JSON.parse(await readFile(new URL('version.json', bundled), 'utf8')), { version: manifest.version })
  for (const file of ['app.js', 'styles.css', 'icon.webp', 'cover.svg']) {
    assert.deepEqual(await readFile(new URL(file, bundled)), await readFile(new URL(file, source)))
  }
  const html = await readFile(new URL('index.html', source), 'utf8')
  const script = await readFile(new URL('app.js', source), 'utf8')
  assert.ok(html.includes('<head>') && html.includes('</head>'))
  assert.doesNotMatch(html, /\sstyle\s*=|<style\b|\son\w+\s*=/i)
  assert.equal([...html.matchAll(/<script\b[^>]*>[\s\S]*?<\/script>/gi)].length, 1)
  assert.match(html, /<script src="app\.js" defer><\/script>/)
  assert.doesNotMatch(script, /\binnerHTML\b|\bouterHTML\b|insertAdjacentHTML|\.style\b|setAttribute\(\s*['"]style['"]|\sstyle\s*=/i)
  const embedded = await readFile(new URL('index.html', bundled), 'utf8')
  assert.match(embedded, /default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self'; connect-src https:\/\/search\.xiangban\.xyz; base-uri 'none'; form-action 'none'/)
  assert.doesNotMatch(embedded, /unsafe-inline/)
  assert.match(script, /getComputedStyle\(\$\('\.masthead'\)\)\.display === 'none'/)
  const cover = await readFile(new URL('cover.svg', source), 'utf8')
  assert.match(cover, /width="640" height="360" viewBox="0 0 640 360"/)
  assert.doesNotMatch(cover, /feDropShadow/)
})

test('wancisou opens the bundle only when its latest published version matches', () => {
  const detail = {
    kind: 'app', slug: 'wancisou', status: 'published', network_disabled: false, version: 4,
    versions: [
      { version: 3, manifest: { id: 'wancisou', version: '1.0.2' } },
      { version: 4, manifest: { id: 'wancisou', version: '2.0.0' } },
    ],
  }
  assert.equal(canOpenBundledWancisou(detail, '2.0.0'), true)
  for (const version of ['1.0.2', '', null, undefined, 2]) assert.equal(canOpenBundledWancisou(detail, version), false)
  assert.equal(canOpenBundledWancisou({ ...detail, network_disabled: true }, '2.0.0'), false)
  assert.equal(canOpenBundledWancisou({ ...detail, network_disabled: undefined }, '2.0.0'), false)
  assert.equal(canOpenBundledWancisou({ ...detail, status: 'removed' }, '2.0.0'), false)
  assert.equal(canOpenBundledWancisou({ ...detail, slug: 'other' }, '2.0.0'), false)
  assert.equal(canOpenBundledWancisou({ ...detail, kind: 'game' }, '2.0.0'), false)
  assert.equal(canOpenBundledWancisou({ ...detail, version: 5 }, '2.0.0'), false)
})
