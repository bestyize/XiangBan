import { spawn, spawnSync } from 'node:child_process'
import { mkdir, readFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import path from 'node:path'
import assert from 'node:assert/strict'
const root = fileURLToPath(new URL('../', import.meta.url))
await mkdir(path.join(root, 'artifacts'), { recursive: true })
const report = path.join(root, `artifacts/games-smoke-${Date.now()}.json`)
const defaultBinary = process.platform === 'win32' ? 'src-tauri/target/release/xiaoban-desktop-wallpaper.exe' : 'src-tauri/target/release/xiaoban-desktop-wallpaper'
const startupId = `${Date.now()}${process.pid}`
const startupName = `XiaoBanStartupSmoke-${startupId}`
const careSmoke = process.env.XIAOBAN_CARE_SMOKE === '1'
const uiSmoke = process.env.XIAOBAN_UI_SMOKE === '1'
const childEnv = { ...process.env, XIAOBAN_GAMES_SMOKE: '1', XIAOBAN_SMOKE_REPORT: report, XIAOBAN_TEST_STARTUP_ID: startupId }
if (careSmoke) childEnv.XIAOBAN_CARE_SMOKE = '1'
else delete childEnv.XIAOBAN_CARE_SMOKE
if (uiSmoke) childEnv.XIAOBAN_UI_SMOKE = '1'
else delete childEnv.XIAOBAN_UI_SMOKE
const child = spawn(process.argv[2] ?? path.join(root, defaultBinary), ['--smoke-test', '--autostart'], { env: childEnv, stdio: ['ignore', 'pipe', 'pipe'] })
let output = ''
child.stdout.on('data', d => { output = (output + d).slice(-14000) })
child.stderr.on('data', d => { output = (output + d).slice(-14000) })
let timedOut = false
const timeout = setTimeout(() => { timedOut = true; child.kill() }, 200000)
function cleanupTestStartup() {
  if (process.platform !== 'win32') return
  if (!/^XiaoBanStartupSmoke-\d+$/.test(startupName)) throw new Error('拒绝清理非本次数字测试启动项')
  const script = `$ErrorActionPreference = 'Stop'
$targets = @(
  @{ Path = 'Software\\Microsoft\\Windows\\CurrentVersion\\Run'; Name = '${startupName}' },
  @{ Path = 'Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run'; Name = '${startupName}' }
)
try {
  foreach ($target in $targets) {
    $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey($target.Path, $true)
    if ($null -eq $key) { continue }
    try { $key.DeleteValue($target.Name, $false) } finally { $key.Dispose() }
  }
} catch {
  [Console]::Error.WriteLine($_.Exception.Message)
  exit 1
}
exit 0`
  const encoded = Buffer.from(script, 'utf16le').toString('base64')
  const result = spawnSync('powershell.exe', ['-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-EncodedCommand', encoded], { encoding: 'utf8', windowsHide: true })
  if (result.error) throw result.error
  if (result.status !== 0) throw new Error(`清理本次测试启动项失败：${(result.stderr ?? '').trim() || `PowerShell exit ${result.status}`}`)
}
try {
  await new Promise((resolve, reject) => { child.on('error', reject); child.on('exit', (code, signal) => code === 0 && !timedOut ? resolve() : reject(new Error(`${timedOut ? 'Game smoke timed out' : 'Game smoke failed'}: ${code ?? signal}\n${output}`))) })
  const result = JSON.parse(await readFile(report, 'utf8'))
  const game = result.games?.Ok
  assert.ok(game, JSON.stringify(result.games))
  assert.deepEqual([...game.frontend.ready].sort(), ['falling-blocks', 'merge-2048', 'block-puzzle', 'water-sort', 'snake-arena', 'ink-blade', 'dusk-glider'].sort())
  assert.ok(Object.values(game.frontend.security).every(v => v === true))
  for (const key of ['backgroundSuspended', 'settingsPreserved', 'positionRestored', 'closed']) assert.equal(game[key], true, key)
  assert.equal(game.frontend.saveRoundTrip, true)
  assert.equal(game.frontend.controlRejected, true)
  assert.equal(game.frontend.escapeReturned, true)
  if (uiSmoke) {
    const ui = result.ui?.Ok
    assert.ok(ui, `UI smoke did not pass: ${JSON.stringify(result.ui)}`)
    assert.equal(ui.ok, true, 'ui.ok')
    assert.equal(ui.configBound, true, 'ui.configBound')
    assert.equal(ui.marker, 'vue-native-control-readonly', 'ui.marker')
    assert.deepEqual([...ui.pages].sort(), ['wallpapers', 'companion', 'games', 'care', 'settings'].sort(), 'ui.pages')
    assert.equal(ui.gamePackCount, 7, 'ui.gamePackCount')
  }
  if (careSmoke) {
    const care = result.care?.Ok
    assert.ok(care, `Care smoke did not pass: ${JSON.stringify(result.care)}`)
    for (const key of ['persistence', 'previewNoTask', 'bubbleReady', 'noFocus', 'dismiss', 'snooze', 'disableCancels', 'delete', 'gameDeferred', 'gameResumeSummary', 'expiry', 'restartDedup']) assert.equal(care[key], true, `care.${key}`)
    assert.equal(care.weatherNetworkUsed, false, 'care.weatherNetworkUsed')
  }
  console.log(`PASS: seven offline packs ready, opaque iframe IPC boundary, local save, background suspension and restore${careSmoke ? ', plus care delivery and persistence' : ''}${uiSmoke ? ', plus native control UI report' : ''}. ${report}`)
} finally {
  clearTimeout(timeout)
  cleanupTestStartup()
}
