import { useEffect, useRef, useState } from 'react'
import { useLocation } from 'wouter'
import { quizAccountId } from '../quizzes/progress.ts'
import { useSession } from '../lib/session.tsx'
import { useLoginModal } from '../lib/loginModal.ts'
import { rememberLoginReturn } from '../lib/loginReturn.ts'
import { Robot } from './Robot.tsx'

const DISMISSED = 'xb_login_reminder_seen'
let seen = false
function dismissed() { try { return seen || sessionStorage.getItem(DISMISSED) === '1' } catch { return seen } }
function dismiss() { seen = true; try { sessionStorage.setItem(DISMISSED, '1') } catch {} }

export function useLoginReminder(kind: 'quiz' | 'game') {
  const { isLoggedIn, status } = useSession()
  const [location] = useLocation()
  const { openLogin } = useLoginModal()
  const [open, setOpen] = useState(false)
  const next = useRef<(() => void) | null>(null)
  const dialog = useRef<HTMLDialogElement>(null)
  useEffect(() => {
    if (open) dialog.current?.showModal()
    else dialog.current?.close()
  }, [open])
  function proceed() { dismiss(); setOpen(false); next.current?.(); next.current = null }
  function ask(action: () => void) {
    if (isLoggedIn || (status === 'loading' && quizAccountId() !== null) || dismissed()) return action()
    next.current = action; setOpen(true)
  }
  const modal = <dialog ref={dialog} className="login-reminder" aria-labelledby="login-reminder-title" onCancel={(event) => { event.preventDefault(); proceed() }}>
    <div className="login-reminder-handle" aria-hidden="true" />
    <div className="flex justify-center"><Robot spec="happy" label="小伴帮你留下记录" size={84} /></div>
    <h2 id="login-reminder-title">给这次体验留个位置</h2>
    <p>{kind === 'quiz' ? '登录后，测试进度和结果会跟着账号保存。换设备，也能接着认识自己。' : '登录后，游戏成绩和记录会跟着账号保存。换设备，也能找回来。'}</p>
    <button type="button" className="login-reminder-primary" onClick={() => {
      dismiss(); rememberLoginReturn({ path: location, start: true }); setOpen(false); openLogin()
    }}>登录 / 注册</button>
    <button type="button" className="login-reminder-skip" onClick={proceed}>先体验一下</button>
  </dialog>
  return { ask, modal }
}
