// 验证会话接续：无界面建立会话 → 按编号恢复 → 记录文件位置 → 单进程多轮输入 → 提示你手动在终端接续。
// 需要已登录 Claude Code。用法：node 04-claude-resume.mjs
import { existsSync, mkdtempSync, readdirSync } from 'node:fs'
import { homedir, tmpdir } from 'node:os'
import { join } from 'node:path'
import readline from 'node:readline/promises'
import { cleanEnv, header, jsonLines, run, save, start, verdict, which } from './lib.mjs'

header('Claude Code 会话接续')
if (!which('claude')) { verdict(false, '没找到 claude 命令'); process.exit(1) }
const dir = mkdtempSync(join(tmpdir(), 'p0-resume-'))
const secret = 'PINEAPPLE-' + Math.random().toString(36).slice(2, 7).toUpperCase()
const summary = { dir, secret }

// 1. 建立会话
const a = await run('claude', ['-p', `记住暗号 ${secret}，只回复 好`, '--output-format', 'json'], { cwd: dir, timeoutMs: 120000 })
let first = {}
try { first = JSON.parse(a.stdout) } catch {}
summary.sessionId = first.session_id ?? null
verdict(!!summary.sessionId, `建立会话，session_id=${summary.sessionId}（用时 ${a.ms} ms）`)

// 2. 按编号恢复（换一个目录也要能找到）
if (summary.sessionId) {
  const b = await run('claude', ['-p', '--resume', summary.sessionId, '刚才的暗号是什么？只回复暗号', '--output-format', 'json'], { cwd: dir, timeoutMs: 120000 })
  let second = {}
  try { second = JSON.parse(b.stdout) } catch {}
  summary.resumeSameSession = second.session_id === summary.sessionId
  summary.resumeRemembered = String(second.result ?? '').includes(secret)
  verdict(summary.resumeRemembered, `恢复后记得暗号（${JSON.stringify(second.result)}），用时 ${b.ms} ms`)
  verdict(summary.resumeSameSession, `恢复后仍是同一个 session_id`)

  // 3. 会话记录文件
  const configDir = process.env.CLAUDE_CONFIG_DIR ?? join(homedir(), '.claude')
  const projectDirName = dir.replace(/[^a-zA-Z0-9]/g, '-')
  const transcript = join(configDir, 'projects', projectDirName, `${summary.sessionId}.jsonl`)
  summary.transcriptExpected = transcript
  summary.transcriptExists = existsSync(transcript)
  if (!summary.transcriptExists) {
    // 目录名规则可能和文档不同（例如 Windows 盘符），列出实际目录供对照。
    summary.projectsDirListing = existsSync(join(configDir, 'projects')) ? readdirSync(join(configDir, 'projects')).filter(n => n.includes('p0-resume')) : []
  }
  verdict(summary.transcriptExists, `会话记录文件在预期位置：${summary.transcriptExpected}`)

  // 4. 单进程多轮（stream-json 输入）
  const child = start('claude', ['-p', '--input-format', 'stream-json', '--output-format', 'stream-json', '--verbose', '--resume', summary.sessionId], { cwd: dir, env: cleanEnv() })
  let out = ''
  child.stdout.on('data', d => (out += d))
  const sendUser = text => child.stdin.write(JSON.stringify({ type: 'user', message: { role: 'user', content: [{ type: 'text', text }] } }) + '\n')
  const t0 = Date.now()
  sendUser('把暗号重复一遍，只回复暗号')
  const waitResults = async n => { while (jsonLines(out).filter(e => e.type === 'result').length < n && Date.now() - t0 < 120000) await new Promise(r => setTimeout(r, 300)) }
  await waitResults(1)
  const t1 = Date.now()
  sendUser('再把暗号倒过来写，只回复结果')
  await waitResults(2)
  child.stdin.end()
  child.kill()
  const results = jsonLines(out).filter(e => e.type === 'result')
  summary.streamInput = { results: results.map(r => r.result), firstTurnMs: t1 - t0, secondTurnMs: Date.now() - t1 }
  verdict(results.length >= 2, `单进程多轮：收到 ${results.length} 个结果，第二轮用时 ${summary.streamInput.secondTurnMs} ms`)
  save('claude-streaminput-events.jsonl', out)

  // 5. 手动确认：在真实终端里接着聊
  console.log(`\n请手动验证（这一步脚本无法代替你）：\n  1) 打开一个新终端，执行：\n     cd ${dir}\n     claude --resume ${summary.sessionId}\n  2) 确认历史里能看到上面的几轮对话，并能继续输入。\n`)
  if (process.stdin.isTTY) {
    const rl = readline.createInterface({ input: process.stdin, output: process.stdout })
    const ans = (await rl.question('终端里能看到完整历史并接着聊吗？(y/n/跳过=回车) ')).trim().toLowerCase()
    rl.close()
    summary.manualTerminalResume = ans === 'y' ? true : ans === 'n' ? false : null
  }
}
save('claude-resume-summary.json', summary)
