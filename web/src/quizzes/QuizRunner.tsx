import { useEffect, useRef, useState } from 'react'
import { Link, useLocation } from 'wouter'
import { clearAttempt, loadAttempt, newAttempt, saveAttempt } from './progress.ts'
import { isComplete } from './scoring.ts'
import { useLoginReminder } from '../components/LoginReminder.tsx'
import { readLoginReturn, clearLoginReturn } from '../lib/loginReturn.ts'
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
  const heading = useRef<HTMLHeadingElement>(null)
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
    if (intent?.path === `/tests/${quiz.id}` && intent.start) { setStarted(true); clearLoginReturn() }
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

  function go(index: number) {
    setAttempt((previous) => ({ ...previous, cursor: index }))
    setMessage('')
    setOverview(false)
  }

  function choose(value: number) {
    setAttempt((previous) => {
      const answers = [...previous.answers]
      answers[previous.cursor] = value
      return { ...previous, answers, completedAt: null }
    })
    setMessage('')
  }

  function restart() {
    if (answered > 0 && !window.confirm('要清空这套测试的答案，重新开始吗？')) return
    clearAttempt(quiz)
    setAttempt(newAttempt(quiz))
    setStarted(true)
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
    navigate(`/tests/${quiz.id}/report`)
  }

  if (!started) return <main className={`quiz-shell quiz-theme-${quiz.id}`}>
    <header className="quiz-header"><Link href="/tests" className="quiz-back">← 测试合集</Link><span className="quiz-mini-brand">下班了么</span></header>
    <div className="quiz-intro">
      <span className="quiz-eyebrow">{quiz.eyebrow}</span>
      <QuizArt kind={quiz.id} />
      <h1>{quiz.name}<span>{quiz.subtitle}</span></h1>
      <p className="quiz-intro-description">{quiz.description}</p>
      <div className="quiz-intro-facts"><span>{total} 道选择题</span><span>{quiz.duration}</span><span>无需登录</span></div>
      {quiz.id === 'mbti' && <p className="quiz-subtle">MBTI 风格探索 · 非官方量表 · 结果仅供参考</p>}
      <div className="quiz-paper"><h2>把答案留给真实的你</h2><p>{quiz.instructions}</p><p className="quiz-subtle">{isLoggedIn ? '进度和报告会保存到账号，换设备也能找回。' : '进度暂存在当前浏览器。登录后可保存到账号，换设备接着答。'}</p></div>
      {attempt.completedAt ? <>
        <p className="quiz-resume-note">这套测试你已完成，报告还在这里。</p>
        <Link className="quiz-button" href={`/tests/${quiz.id}/report`}>查看我的报告 <span aria-hidden="true">↗</span></Link>
        <button type="button" className="quiz-text-button" onClick={() => ask(() => setStarted(true))}>回看 / 修改答案</button>
        <button type="button" className="quiz-text-button" onClick={() => ask(restart)}>重新测一次</button>
      </> : <>
        {answered > 0 && <p className="quiz-resume-note">已经答了 {answered} / {total} 题，接着上次的进度。</p>}
        <button type="button" className="quiz-button" onClick={() => ask(() => setStarted(true))}>{answered > 0 ? '继续测试' : '开始测试'} <span aria-hidden="true">→</span></button>
        {answered > 0 && <button type="button" className="quiz-text-button" onClick={() => ask(restart)}>重新开始</button>}
      </>}
      <details className="quiz-method"><summary>题目来源与结果说明</summary><p>{quiz.caveat}</p>{quiz.id === 'mbti' && <p>公共领域题目参考 <a href="https://ipip.ori.org/newBigFive5broadKey.htm" target="_blank" rel="noreferrer">IPIP 条目</a>，四组偏好框架参考 Myers &amp; Briggs Foundation；其余题目与报告为原创。</p>}</details>
    </div>
    {modal}
  </main>

  return <main className={`quiz-shell quiz-runner quiz-theme-${quiz.id}`}>
    <header className="quiz-header"><Link href="/tests" className="quiz-back">← 测试合集</Link><span className="quiz-mini-brand">{quiz.name}</span></header>
    <div className="quiz-progress-area">
      <div className="quiz-progress-label"><span>已完成 {answered} / {total}</span><button className="quiz-text-button" type="button" aria-expanded={overview} aria-controls="quiz-overview" onClick={() => setOverview(!overview)}>答题一览</button></div>
      <div className="quiz-progress-track" role="progressbar" aria-label="答题进度" aria-valuemin={0} aria-valuemax={total} aria-valuenow={answered}><span style={{ width: `${answered / total * 100}%` }} /></div>
    </div>
    {overview && <section className="quiz-overview" id="quiz-overview" aria-label="选择题号回看答案">
      <p>实心为已答，点击题号可回看修改。</p>
      <div>{quiz.questions.map((item, i) => <button key={item.id} type="button" aria-label={`第 ${i + 1} 题，${attempt.answers[i] === null ? '未作答' : '已作答'}`} aria-current={i === attempt.cursor ? 'step' : undefined} className={attempt.answers[i] === null ? '' : 'is-answered'} onClick={() => go(i)}>{i + 1}</button>)}</div>
    </section>}
    <section className="quiz-question" key={question.id}>
      <span className="quiz-eyebrow">QUESTION {String(attempt.cursor + 1).padStart(2, '0')} <span>/ {total}</span></span>
      <h1 ref={heading} tabIndex={-1} id="quiz-question-title">{question.prompt}</h1>
      <p className="quiz-question-hint">{quiz.id === 'mbti' ? '这句话有多像平时的你？' : quiz.persona ? '哪一种状态，让你觉得最像自己？' : '哪一句最像你的内心弹幕？'}</p>
      <fieldset className="quiz-options" aria-labelledby="quiz-question-title">
        <legend className="quiz-sr-only">请选择一个答案</legend>
        {question.options.map((option, i) => <label className={`quiz-option ${selected === option.value ? 'is-selected' : ''}`} key={option.value}>
          <input type="radio" name={question.id} value={option.value} checked={selected === option.value} onChange={() => choose(option.value)} />
          <span className="quiz-option-key" aria-hidden="true">{quiz.id === 'mbti' ? String(i + 1).padStart(2, '0') : String.fromCharCode(65 + i)}</span>
          <span className="quiz-option-text">{option.label}</span>
          <span className="quiz-option-check" aria-hidden="true">{selected === option.value && <svg viewBox="0 0 16 16" fill="none"><path d="m3.5 8 3 3 6-6" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" /></svg>}</span>
        </label>)}
      </fieldset>
      <p className="quiz-message" role="status">{message}</p>
    </section>
    <footer className="quiz-question-footer">
      <div className="quiz-question-navigation">
        <button className="quiz-button quiz-button-outline" type="button" disabled={attempt.cursor === 0} onClick={() => go(attempt.cursor - 1)}>上一题</button>
        <button className="quiz-button" type="button" disabled={selected === null} onClick={() => attempt.cursor === total - 1 ? finish() : go(attempt.cursor + 1)}>{attempt.cursor === total - 1 ? '生成我的报告' : '下一题'} <span aria-hidden="true">→</span></button>
      </div>
      <p>答案会自动保存 · 可以回看修改</p>
    </footer>
    {modal}
  </main>
}
