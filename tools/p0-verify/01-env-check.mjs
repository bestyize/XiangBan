// 收集环境信息：系统、Node、git、Claude Code、Codex、两个桌面版是否在运行。不读取任何登录凭据。
import { platform, release, arch } from 'node:os'
import { header, run, save, which, isWin } from './lib.mjs'

header('环境检查')
const info = { os: `${platform()} ${release()} ${arch()}`, node: process.version }
for (const [name, cmd, args] of [
  ['git', 'git', ['--version']],
  ['claude', 'claude', ['--version']],
  ['codex', 'codex', ['--version']],
]) {
  const path = which(cmd)
  info[name] = { path, version: path ? (await run(cmd, args, { timeoutMs: 20000 })).stdout.trim() : null }
  console.log(`${path ? '✅' : '❌'} ${name}: ${info[name].version ?? '未找到'}`)
}

// 登录状态只看命令是否成功，不保存输出里可能有的账号信息。
for (const [name, args] of [['claude', ['auth', 'status']], ['codex', ['login', 'status']]]) {
  if (!info[name].path) continue
  const r = await run(name, args, { timeoutMs: 20000 })
  info[name].loggedIn = r.code === 0
  console.log(`${r.code === 0 ? '✅' : '⚠️'} ${name} 登录状态命令退出码 ${r.code}`)
}

// 桌面版是否在运行（只看进程名）。
const ps = isWin
  ? (await run('tasklist', ['/fo', 'csv', '/nh'], { timeoutMs: 20000 })).stdout
  : (await run('ps', ['-axo', 'comm'], { timeoutMs: 20000 })).stdout
// macOS 的 ps 输出完整路径，认 .app；Windows 的 tasklist 认 .exe。命令行工具不算。
info.desktopApps = isWin
  ? { claude: /^"Claude\.exe"/im.test(ps), chatgpt: /^"ChatGPT\.exe"/im.test(ps), codex: /^"Codex\.exe"/im.test(ps) }
  : { claude: /Claude\.app\//.test(ps), chatgpt: /ChatGPT\.app\//.test(ps), codex: /Codex\.app\//.test(ps) }
console.log('桌面版进程（名字匹配，仅供参考）：', info.desktopApps)
save('env.json', info)
