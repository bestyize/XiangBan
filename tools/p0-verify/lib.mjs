// 公共函数：跨平台找命令、运行子进程、写结果。无第三方依赖，Node 18+。
import { spawn, spawnSync } from 'node:child_process'
import { mkdirSync, writeFileSync, appendFileSync } from 'node:fs'
import { homedir, platform } from 'node:os'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

export const isWin = platform() === 'win32'
export const OUT = join(dirname(fileURLToPath(import.meta.url)), 'out')
mkdirSync(OUT, { recursive: true })

/** 把家目录替换成 ~，避免结果文件泄露用户名。 */
export function redact(text) {
  const home = homedir()
  return String(text).split(home).join('~').split(home.replaceAll('\\', '/')).join('~')
}

export function save(name, data) {
  const body = typeof data === 'string' ? data : JSON.stringify(data, null, 2)
  writeFileSync(join(OUT, name), redact(body))
  console.log(`  已写入 out/${name}`)
}
export function append(name, line) {
  appendFileSync(join(OUT, name), redact(line) + '\n')
}

/** 找到可执行文件的完整路径；找不到返回 null。 */
export function which(cmd) {
  const r = spawnSync(isWin ? 'where' : 'which', [cmd], { encoding: 'utf8' })
  if (r.status !== 0) return null
  return r.stdout.split(/\r?\n/).filter(Boolean)[0] ?? null
}

function winQuote(arg) {
  return /[\s"&|<>^()%!]/.test(arg) ? `"${arg.replaceAll('"', '\\"')}"` : arg
}

/** 启动子进程。Windows 上 npm 装的命令是 .cmd 垫片，必须经 shell 运行，参数手动加引号。 */
export function start(cmd, args, opts = {}) {
  const full = which(cmd) ?? cmd
  const needShell = isWin && /\.(cmd|bat)$/i.test(full)
  if (needShell) return spawn(winQuote(full), args.map(winQuote), { ...opts, shell: true, windowsHide: true })
  return spawn(full, args, { ...opts, windowsHide: true })
}

/** 运行到结束，收集输出；超时则杀掉。 */
/** 从 Claude Code 里面运行本脚本时，会继承它的会话编号，导致子进程复用同一会话；这里去掉。 */
export function cleanEnv(extra = {}) {
  const env = { ...process.env, ...extra }
  delete env.CLAUDE_CODE_SESSION_ID
  delete env.CLAUDE_CODE_CHILD_SESSION
  delete env.CLAUDE_CODE_REMOTE_SESSION_ID
  return env
}

export function run(cmd, args, { cwd, input, timeoutMs = 120000, env } = {}) {
  return new Promise(resolve => {
    const t0 = Date.now()
    const child = start(cmd, args, { cwd, env: cleanEnv(env) })
    let stdout = ''
    let stderr = ''
    const timer = setTimeout(() => child.kill(), timeoutMs)
    child.stdout.on('data', d => (stdout += d))
    child.stderr.on('data', d => (stderr += d))
    child.on('error', e => resolve({ code: -1, stdout, stderr: String(e), ms: Date.now() - t0, timedOut: false }))
    child.on('close', code => {
      clearTimeout(timer)
      resolve({ code, stdout, stderr, ms: Date.now() - t0, timedOut: Date.now() - t0 >= timeoutMs })
    })
    if (input !== undefined) child.stdin.end(input)
    else child.stdin.end()
  })
}

export function jsonLines(text) {
  return text.split(/\r?\n/).filter(Boolean).map(l => { try { return JSON.parse(l) } catch { return { _unparsed: l } } })
}

export function header(title) { console.log(`\n=== ${title} ===`) }
export function verdict(ok, text) { console.log(`${ok ? '✅' : '❌'} ${text}`); return ok }
