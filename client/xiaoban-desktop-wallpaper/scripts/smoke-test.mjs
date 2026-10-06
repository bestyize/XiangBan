import { spawn } from 'node:child_process'
import { mkdir, readFile, writeFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import path from 'node:path'
import assert from 'node:assert/strict'

if (process.platform !== 'darwin') throw new Error('这个脚本需要在 macOS 上执行')
const root = fileURLToPath(new URL('../', import.meta.url))
const binary = process.argv[2] || path.join(root, 'src-tauri/target/aarch64-apple-darwin/release/bundle/macos/小伴.app/Contents/MacOS/xiaoban-desktop-wallpaper')
await mkdir(path.join(root, 'artifacts'), { recursive: true })
const report = path.join(root, `artifacts/smoke-macos-${Date.now()}.json`)
const child = spawn(binary, ['--smoke-test'], {
  env: { ...process.env, XIAOBAN_SMOKE_REPORT: report }, stdio: ['ignore', 'pipe', 'pipe'],
})
let output = ''
child.stdout.on('data', data => { output = (output + data).slice(-16000) })
child.stderr.on('data', data => { output = (output + data).slice(-16000) })
const timeout = setTimeout(() => { child.kill('SIGTERM') }, 90000)
try {
  await new Promise((resolve, reject) => {
    child.on('error', reject)
    child.on('exit', (code, signal) => code === 0 ? resolve() : reject(new Error(`macOS 原型退出异常 ${code ?? signal}\n${output}`)))
  })
  const raw = await readFile(report, 'utf8')
  await writeFile(path.join(root, 'artifacts/smoke-macos.json'), raw)
  const result = JSON.parse(raw)
  assert.equal(result.startupResult.Ok, 'ok', JSON.stringify(result.startupResult))
  assert.equal(result.wallpaperResult.Ok, 'ok', JSON.stringify(result.wallpaperResult))
  assert.equal(result.floatingResult.Ok, 'ok', JSON.stringify(result.floatingResult))
  for (const scene of [result.floatingBefore.floating, result.wallpaper.wallpaper, result.floatingAfter.floating]) {
    assert.ok(scene?.visible, 'AppKit 窗口未显示 / 前端未完成挂载')
    assert.ok(scene.frontendReady, '前端没有报告挂载完成')
    assert.ok(scene.width > 0 && scene.height > 0)
    assert.ok(scene.joinsAllSpaces, '窗口未配置桌面空间行为')
  }
  const wallpaper = result.wallpaper.wallpaper
  assert.ok(wallpaper.level > wallpaper.desktopLevel && wallpaper.level < wallpaper.desktopIconLevel, '壁纸不在桌面背景和图标之间')
  assert.ok(wallpaper.ignoresMouseEvents, '壁纸拦截了桌面输入')
  assert.equal(result.floatingBefore.floating.opaque, false, '悬浮窗不透明')
  assert.equal(result.wallpaper.floating, null, '进入壁纸后残留悬浮窗')
  assert.equal(result.floatingAfter.wallpaper, null, '返回悬浮后残留壁纸窗')
  console.log(`PASS: macOS 悬浮 → 桌面背景层壁纸 → 悬浮。报告：${report}`)
} finally { clearTimeout(timeout) }
