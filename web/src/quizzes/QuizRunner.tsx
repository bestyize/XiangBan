import { useEffect, useRef, useState } from 'react'
import { Link, useLocation } from 'wouter'
import { clearAttempt, loadAttempt, newAttempt, saveAttempt } from './progress.ts'
import { isComplete } from './scoring.ts'
import { useLoginReminder } from '../components/LoginReminder.tsx'
import { NavBar } from '../components/SubPage.tsx'
import { readLoginReturn, clearLoginReturn } from '../lib/loginReturn.ts'
import { track } from '../lib/track.ts'
import { useSession } from '../lib/session.tsx'
import { QuizArt } from './QuizArt.tsx'
import type { QuizAttempt, QuizDefinition } from './types.ts'

export function QuizRunner({ quiz }: { quiz: QuizDefinition }) {
  const [attempt, setAttempt] = useState<QuizAttempt>(() => loadAttempt(quiz) ?? newAttempt(quiz))
  const { ask, modal } = useLoginReminder('quiz')
  const { isLoggedIn } = useSession()
  const [started, setStarted] = useState(false)
  const [overview, setOverview] = useState(false)
  const [message, setMessage] = useState('')
  const [direction, setDirection] = useState<'forward' | 'back'>('forward')
  const heading = useRef<HTMLHeadingElement>(null)
  const advanceTimer = useRef<number | null>(null)
  /** Which option a finger/mouse is pressing; keyboard picks never auto-advance, so arrowing through options stays possible. */
  const pointerPick = useRef<number | null>(null)
  const [, navigate] = useLocation()
  const total = quiz.questions.length
  const answered = attempt.answers.filter((answer) => answer !== null).length
  const question = quiz.questions[attempt.cursor]
  const selected = attempt.answers[attempt.cursor]

  useEffect(() => {
    const update = () => {
      const next = loadAttempt(quiz)
      if (next) setAttempt((previous) => JSON.stringify(previous) === JSON.stringify(next) ? previous : next)
    }
    window.addEventListener('xb-quiz-progress', update)
    return () => window.removeEventListener('xb-quiz-progress', update)
  }, [quiz])

  useEffect(() => {
    const intent = readLoginReturn()
    if (intent?.path === `/tests/${quiz.id}` && intent.start) { start('new'); clearLoginReturn() }
  }, [quiz.id])

  useEffect(() => {
    if (started) saveAttempt(quiz, attempt)
  }, [quiz, attempt, started])

  useEffect(() => {
    if (started) {
      heading.current?.focus({ preventScroll: true })
      window.scrollTo({ top: 0, behavior: 'instant' })
    }
  }, [attempt.cursor, started])

  useEffect(() => () => { if (advanceTimer.current) clearTimeout(advanceTimer.current) }, [])

  function cancelAdvance() {
    if (advanceTimer.current) clearTimeout(advanceTimer.current)
    advanceTimer.current = null
  }

  function go(index: number) {
    cancelAdvance()
    setDirection(index < attempt.cursor ? 'back' : 'forward')
    setAttempt((previous) => ({ ...previous, cursor: index }))
    setMessage('')
    setOverview(false)
  }

  /** Tapping an answer moves on by itself after a beat, long enough to see the pick land. */
  function advanceSoon() {
    cancelAdvance()
    const from = attempt.cursor
    if (from >= total - 1) return
    advanceTimer.current = window.setTimeout(() => {
      advanceTimer.current = null
      setDirection('forward')
      setAttempt((previous) => previous.cursor === from ? { ...previous, cursor: from + 1 } : previous)
    }, 320)
  }

  function choose(value: number) {
    const tapped = pointerPick.current === value
    pointerPick.current = null
    setAttempt((previous) => {
      const answers = [...previous.answers]
      answers[previous.cursor] = value
      return { ...previous, answers, completedAt: null }
    })
    setMessage('')
    if (tapped) advanceSoon()
    else cancelAdvance()
  }

  function start(mode: 'new' | 'resume' | 'review' | 'retake') {
    track('quiz_start', { quiz: quiz.id, mode })
    setStarted(true)
  }

  function restart() {
    if (answered > 0 && !window.confirm('要清空这套测试的答案，重新开始吗？')) return
    clearAttempt(quiz)
    setAttempt(newAttempt(quiz))
    start('retake')
    setMessage('')
    setOverview(false)
  }

  function finish() {
    if (!isComplete(quiz, attempt.answers)) {
      const missing = attempt.answers.findIndex((answer) => answer === null)
      go(missing)
      setMessage(`还有题目没答，先完成第 ${missing + 1} 题。`)
      return
    }
    const complete = { ...attempt, completedAt: Date.now() }
    saveAttempt(quiz, complete)
    track('quiz_complete', { quiz: quiz.id })
    navigate(`/tests/${quiz.id}/report`)
  }

  if (!started) return <>
    <main className={`quiz-shell app-subpage quiz-intro-page quiz-theme-${quiz.id}`}>
      <NavBar title={quiz.name} back="/tests" />
      <div className="quiz-intro">
        <QuizArt kind={quiz.id} />
        <span className="quiz-eyebrow">{quiz.eyebrow}</span>
        <h1>{quiz.name}<span>{quiz.subtitle}</span></h1>
        <div className="quiz-intro-facts"><span>{total} 道题</span><span>{quiz.duration}</span>{isLoggedIn && <span>进度存在账号里</span>}</div>
        <p className="quiz-intro-description">{quiz.description}</p>
        <p className="quiz-intro-note">{quiz.instructions}</p>
        {quiz.id === 'mbti' && <p className="quiz-subtle">MBTI 风格探索 · 非官方量表 · 结果仅供参考</p>}
        <details className="quiz-method"><summary>题目来源与结果说明</summary><p>{quiz.caveat}</p>{quiz.id === 'mbti' && <p>公共领域题目参考 <a href="https://ipip.ori.org/newBigFive5broadKey.htm" target="_blank" rel="noreferrer">IPIP 条目</a>，四组偏好框架参考 Myers &amp; Briggs Foundation；其余题目与报告为原创。</p>}</details>
      </div>
    </main>
    {/* Outside <main>: its slide-in transform would otherwise make this fixed bar ride along with the page. */}
    <footer className={`quiz-action-bar quiz-theme-${quiz.id}`}>
      {attempt.completedAt ? <>
        <Link className="quiz-button" href={`/tests/${quiz.id}/report`}>查看我的报告</Link>
        <div className="quiz-action-links">
          <button type="button" className="quiz-text-button" onClick={() => ask(() => start('review'))}>回看答案</button>
          <button type="button" className="quiz-text-button" onClick={() => ask(restart)}>重新测一次</button>
        </div>
      </> : <>
        <button type="button" className="quiz-button" onClick={() => ask(() => start(answered > 0 ? 'resume' : 'new'))}>
          {answered > 0 ? <>继续测试<span className="quiz-button-note">已答 {answered}/{total}</span></> : '开始测试'}
        </button>
        {answered > 0 && <div className="quiz-action-links"><button type="button" className="quiz-text-button" onClick={() => ask(restart)}>重新开始</button></div>}
      </>}
    </footer>
    {modal}
  </>

  const last = attempt.cursor === total - 1
  return <main className={`quiz-shell quiz-runner quiz-theme-${quiz.id}`}>
    <NavBar title={quiz.name} back="/tests" />
    <div className="quiz-progress-area">
      <div className="quiz-progress-label"><span>第 <strong>{attempt.cursor + 1}</strong> / {total} 题</span><button className="quiz-text-button" type="button" aria-expanded={overview} aria-controls="quiz-overview" onClick={() => setOverview(!overview)}>答题一览</button></div>
      <div className="quiz-progress-track" role="progressbar" aria-label={`已答 ${answered} / ${total} 题`} aria-valuemin={0} aria-valuemax={total} aria-valuenow={answered}><span style={{ width: `${answered / total * 100}%` }} /></div>
    </div>
    {overview && <section className="quiz-overview" id="quiz-overview" aria-label="选择题号回看答案">
      <p>实心为已答，点击题号可回看修改。</p>
      <div>{quiz.questions.map((item, i) => <button key={item.id} type="button" aria-label={`第 ${i + 1} 题，${attempt.answers[i] === null ? '未作答' : '已作答'}`} aria-current={i === attempt.cursor ? 'step' : undefined} className={attempt.answers[i] === null ? '' : 'is-answered'} onClick={() => go(i)}>{i + 1}</button>)}</div>
    </section>}
    <section className={`quiz-question is-${direction}`} key={question.id}>
      <h1 ref={heading} tabIndex={-1} id="quiz-question-title">{question.prompt}</h1>
      <p className="quiz-question-hint">{quiz.id === 'mbti' ? '这句话有多像平时的你？' : quiz.persona ? '哪一种状态，让你觉得最像自己？' : '哪一句最像你的内心弹幕？'}</p>
      <fieldset className="quiz-options" aria-labelledby="quiz-question-title" onKeyDown={() => { pointerPick.current = null }}>
        <legend className="quiz-sr-only">请选择一个答案</legend>
        {question.options.map((option, i) => <label className={`quiz-option ${selected === option.value ? 'is-selected' : ''}`} key={option.value}
          onPointerDown={() => { pointerPick.current = option.value }}>
          {/* Re-tapping the current answer fires no change event, but should still move on. */}
          <input type="radio" name={question.id} value={option.value} checked={selected === option.value} onChange={() => choose(option.value)}
            onClick={() => { if (selected === option.value && pointerPick.current === option.value) { pointerPick.current = null; advanceSoon() } }} />
          <span className="quiz-option-key" aria-hidden="true">{quiz.id === 'mbti' ? i + 1 : String.fromCharCode(65 + i)}</span>
          <span className="quiz-option-text">{option.label}</span>
          <span className="quiz-option-check" aria-hidden="true"><svg viewBox="0 0 16 16" fill="none"><path d="m3.5 8 3 3 6-6" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" /></svg></span>
        </label>)}
      </fieldset>
      <p className="quiz-message" role="status">{message}</p>
    </section>
    <footer className="quiz-question-footer">
      <div className={`quiz-question-navigation${last ? ' is-final' : ''}`}>
        <button className={`quiz-button quiz-button-outline${attempt.cursor === 0 ? ' is-hidden' : ''}`} type="button" onClick={() => go(attempt.cursor - 1)}>上一题</button>
        {last
          ? <button className="quiz-button" type="button" disabled={selected === null} onClick={finish}>生成我的报告</button>
          // Tapping an answer advances by itself; this is for coming back through answered questions (and keyboard use).
          : <button className={`quiz-button quiz-button-outline${selected === null ? ' is-hidden' : ''}`} type="button" onClick={() => go(attempt.cursor + 1)}>下一题</button>}
      </div>
    </footer>
    {modal}
  </main>
}
