// 探测桌面版应用能否被“送字”：先看输入框能不能被系统无障碍接口找到，再试激活并粘贴。
// 用法：node 05-desktop-probe.mjs <应用名> [tree|paste|both]     例：node 05-desktop-probe.mjs Claude both
// 事先：打开目标应用，进入一个对话页，把光标放在输入框里；macOS 要给运行本脚本的终端开“辅助功能”权限。
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { header, isWin, run, save, verdict } from './lib.mjs'

const [name, modeArg] = [process.argv[2], process.argv[3] ?? 'both']
if (!name) { console.log('用法：node 05-desktop-probe.mjs <应用名: Claude | ChatGPT | Codex> [tree|paste|both]'); process.exit(1) }
const here = dirname(fileURLToPath(import.meta.url))
const modes = modeArg === 'both' ? ['tree', 'paste'] : [modeArg]

header(`桌面版探测：${name}`)
const results = {}
for (const mode of modes) {
  const r = isWin
    ? await run('powershell', ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', join(here, 'probe-win.ps1'), '-Name', name, '-Mode', mode], { timeoutMs: 60000 })
    : await run('osascript', ['-l', 'JavaScript', join(here, 'probe-mac.js'), name, mode], { timeoutMs: 60000 })
  let parsed = null
  try { parsed = JSON.parse((r.stdout || r.stderr).trim().split(/\r?\n/).pop()) } catch {}
  results[mode] = parsed ?? { raw: (r.stdout + r.stderr).slice(0, 1500), code: r.code }
  const ok = parsed?.ok !== false && r.code === 0
  verdict(ok, `${mode}：${ok ? '完成' : '失败'}（${r.ms} ms）`)
  if (mode === 'tree' && parsed?.ok) verdict((parsed.candidateCount ?? 0) > 0, `找到 ${parsed.candidateCount ?? 0} 个可能的输入框`)
  if (mode === 'paste' && parsed?.ok) console.log(`  激活加粘贴用时 ${parsed.activateAndPasteMs} ms；请看应用里是否出现：${parsed.pastedText}`)
  if (parsed?.error) console.log('  ', parsed.error)
}
save(`desktop-probe-${name}.json`, results)
