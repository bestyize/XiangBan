import { useEffect, useMemo, useState } from 'react'
import { Link, Route, Switch, useLocation } from 'wouter'
import { TAB_BAR_SPACE } from '../components/TabBar.tsx'
import { AccountBadge } from '../components/AccountBadge.tsx'
import { CompanionNote } from '../components/CompanionNote.tsx'
import { PageHeader } from '../components/PageHeader.tsx'
import { NavBar } from '../components/SubPage.tsx'
import { findQuiz, findSharedQuiz, quizCatalog } from './catalog.ts'
import { importGuestAttempts, loadAttempt, readSavedAttempt } from './progress.ts'
import { useSession } from '../lib/session.tsx'
import { isComplete, makeReport, scoreAnswers } from './scoring.ts'
import { readSharedScores } from './share.ts'
import { QuizArt } from './QuizArt.tsx'
import { QuizCover } from './QuizCover.tsx'
import { QuizRunner } from './QuizRunner.tsx'
import { ReportShare } from './ReportShare.tsx'
import type { QuizDefinition, QuizReport } from './types.ts'
import './quizzes.css'
import { useDisplaySettings } from '../lib/displaySettings.tsx'
import { visibleDisplayItems } from '../lib/display.ts'

export function QuizHub({ active = true }: { active?: boolean }) {
  const { isLoggedIn, profile } = useSession()
  const { settings, loading: displayLoading } = useDisplaySettings()
  const visibleQuizzes = visibleDisplayItems(quizCatalog, settings.quizzes)
  const [, refresh] = useState(0)
  useEffect(() => { if (active) refresh((value) => value + 1) }, [active])
  useEffect(() => {
    const update = () => refresh((value) => value + 1)
    window.addEventListener('xb-quiz-progress', update)
    return () => window.removeEventListener('xb-quiz-progress', update)
  }, [])
  const localProgress = isLoggedIn && profile ? quizCatalog.filter((quiz) => {
    const guest = readSavedAttempt(quiz, null)
    return guest?.answers.some((answer) => answer !== null) && !readSavedAttempt(quiz, profile.userId)
  }) : []
  return <main className="quiz-shell quiz-hub" style={{ paddingBottom: TAB_BAR_SPACE }}>
    <PageHeader title="趣味测试"><AccountBadge compact /></PageHeader>
    <CompanionNote className="mb-4" spec="curious" name={profile?.companionName || '小伴'}>
      测一测此刻的你？结果只是一面小镜子，我陪你一起看。
    </CompanionNote>
    {localProgress.length > 0 && <div className="mb-4 rounded-2xl border border-lamp/20 bg-lamp/5 p-3 text-xs leading-relaxed text-window/70">
      本机还有 {localProgress.length} 套游客测试进度。
      <button type="button" className="ml-2 min-h-11 text-lamp underline" onClick={() => {
        importGuestAttempts(localProgress, profile!.userId); refresh((value) => value + 1)
      }}>保存到当前账号</button>
    </div>}
    <div className="quiz-catalog" aria-label="趣味测试合集">
      {!displayLoading && visibleQuizzes.map((quiz) => {
        const attempt = loadAttempt(quiz)
        const count = attempt?.answers.filter((answer) => answer !== null).length ?? 0
        return <Link key={quiz.id} href={`/tests/${quiz.id}`} className={`quiz-catalog-card quiz-theme-${quiz.id}`}>
          <div className="quiz-card-cover"><QuizCover kind={quiz.id} /><span className="quiz-card-category">{quiz.category}</span>{count > 0 && <span className="quiz-card-status">{attempt?.completedAt ? '报告已生成' : `已答 ${count} / ${quiz.questions.length}`}</span>}</div>
          <div className="quiz-card-body"><h2>{quiz.name}</h2><p>{quiz.subtitle}</p></div>
          <div className="quiz-card-footer"><span><strong>{quiz.questions.length} 题</strong><small>{quiz.duration}</small></span><span className="quiz-card-arrow" aria-hidden="true">↗</span></div>
        </Link>
      })}
    </div>
    {displayLoading && <p className="py-6 text-sm text-window/60">小伴正在摆好测试…</p>}
    {!displayLoading && visibleQuizzes.length === 0 && <p className="py-6 text-sm text-window/60">小测试正在准备中，过会儿再来看看。</p>}
    <footer className="quiz-hub-footer">结果是一面小镜子。定义你的人，始终是你。</footer>
  </main>
}

function Missing({ report = false, invalid = false }: { report?: boolean; invalid?: boolean }) {
  return <main className="quiz-shell quiz-empty-shell app-subpage">
    <NavBar title="趣味测试" back="/tests" />
    <div className="quiz-empty">
      <h1>{report ? (invalid ? '这份分享链接不完整' : '报告还在等你的答案') : '还没有这套测试'}</h1>
      <p>{report ? (invalid ? '链接可能被截断，向朋友要一份完整链接，或自己测一次吧。' : '先完成一套测试，你的结果名片就会出现在这里。') : '回合集看看，挑一个现在想玩的。'}</p>
      <Link className="quiz-button" href="/tests">去测试合集 →</Link>
    </div>
  </main>
}

function useFragment() {
  const [fragment, setFragment] = useState(() => window.location.hash)
  useEffect(() => {
    const update = () => setFragment(window.location.hash)
    window.addEventListener('hashchange', update)
    return () => window.removeEventListener('hashchange', update)
  }, [])
  return fragment
}

const chevron = <svg className="quiz-fold-chevron" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="m6 9 6 6 6-6" /></svg>

function Report({ quiz, report, scores, shared }: { quiz: QuizDefinition; report: QuizReport; scores: number[]; shared: boolean }) {
  return <>
  <main className={`quiz-shell app-subpage quiz-report quiz-theme-${quiz.id}`}>
    <NavBar title={`${quiz.name} 报告`} back="/tests" />
    {shared && <p className="quiz-shared-note">{quiz.version !== findQuiz(quiz.id)?.version ? '这是朋友分享的 25 题旧版结果。' : '这是朋友分享的结果。'}</p>}
    <section className={`quiz-report-hero${quiz.persona ? ' quiz-character-hero' : ''}`}>
      <div className="quiz-report-top"><span className="quiz-eyebrow">{quiz.eyebrow}</span><span className="quiz-report-seal">此刻<br />名片</span></div>
      <QuizArt kind={quiz.id} code={report.code} />
      <div className="quiz-result-code">{report.code}</div>
      <h1>{report.title}</h1>
      <p className="quiz-result-quote">{report.quote}</p>
      <div className="quiz-tags">{report.tags.map((tag) => <span key={tag}>{tag}</span>)}</div>
      <p className="quiz-report-description">{report.description}</p>
      <div className="quiz-report-hero-footer"><span>此刻的你 · 值得被看见</span><span aria-hidden="true">✦</span></div>
    </section>
    <p className="quiz-report-disclaimer">{quiz.shortCaveat}</p>
    <div className="quiz-folds">
      <details className="quiz-fold" open>
        <summary><span className="quiz-fold-index">01</span><h2>{quiz.metricsTitle}</h2>{chevron}</summary>
        <div className="quiz-fold-body">
          <p className="quiz-subtle">{quiz.metricsIntro}</p>
          {report.metrics.map((metric) => <div className="quiz-metric" key={metric.label}>
            <h3>{metric.label}</h3>
            <div className="quiz-metric-labels"><span>{metric.left} {metric.percent}%</span><span>{metric.right ? `${metric.right} ${100 - metric.percent}%` : ''}</span></div>
            <div className="quiz-metric-track" aria-label={`${metric.label}，${metric.left} ${metric.percent}%${metric.right ? `，${metric.right} ${100 - metric.percent}%` : ''}`}><span style={{ width: `${metric.percent}%` }} /></div>
            <p>{metric.note}</p>
          </div>)}
        </div>
      </details>
      <details className="quiz-fold">
        <summary><span className="quiz-fold-index">02</span><h2>比标签更重要的，是你</h2>{chevron}</summary>
        <div className="quiz-fold-body">
          {report.sections.map((section, i) => <article className="quiz-note" key={section.title}><span className="quiz-note-icon" aria-hidden="true">{['✦', '◐', '↗'][i]}</span><div><h3>{section.title}</h3><p>{section.text}</p></div></article>)}
        </div>
      </details>
      <details className="quiz-fold">
        <summary><span className="quiz-fold-index">03</span><h2>{report.recharge.title}</h2>{chevron}</summary>
        <div className="quiz-fold-body quiz-recharge"><p>{report.recharge.text}</p><span className="quiz-recharge-sign">小伴给你留的，今晚小纸条。</span></div>
      </details>
    </div>
    <div className="quiz-report-links"><Link href={`/tests/${quiz.id}`}>{shared ? '测测我是哪一型' : '回看答案 / 再测一次'}</Link><Link href="/tests">看看其他测试</Link></div>
    <details className="quiz-method"><summary>怎么看待这份结果？</summary><p>{report.caveat}</p></details>
    <footer className="quiz-hub-footer">你可以变化，你不止这一面。</footer>
  </main>
  {/* Outside <main> so the slide-in transform doesn't carry this fixed bar along. */}
  <ReportShare quiz={quiz} report={report} scores={scores} shared={shared} />
  </>
}

function ReportRoute({ id }: { id: string }) {
  const fragment = useFragment()
  const currentQuiz = findQuiz(id)
  const quiz = fragment ? findSharedQuiz(id, new URLSearchParams(fragment.slice(1)).get('v')) : currentQuiz
  const scores = useMemo(() => {
    if (!quiz) return null
    if (fragment) return readSharedScores(quiz, fragment)
    const attempt = loadAttempt(quiz)
    return attempt?.completedAt && isComplete(quiz, attempt.answers) ? scoreAnswers(quiz, attempt.answers) : null
  }, [quiz, fragment])
  const report = useMemo(() => quiz && scores ? makeReport(quiz, scores) : null, [quiz, scores])
  if (!currentQuiz) return <Missing />
  if (!quiz) return <Missing report invalid />
  if (!scores || !report) return <Missing report invalid={Boolean(fragment)} />
  return <Report key={`${id}-${fragment}`} quiz={quiz} report={report} scores={scores} shared={Boolean(fragment)} />
}

function QuizRoute({ id }: { id: string }) {
  const quiz = findQuiz(id)
  return quiz ? <QuizRunner key={quiz.id} quiz={quiz} /> : <Missing />
}

export default function Tests() {
  const [location] = useLocation()
  useEffect(() => {
    const previous = document.title
    return () => { document.title = previous }
  }, [])
  useEffect(() => {
    const quiz = findQuiz(location.split('/')[2] ?? '')
    document.title = `${quiz ? `${quiz.name}${location.endsWith('/report') ? ' · 测试报告' : ' · 趣味测试'}` : '趣味测试'}｜相伴`
    window.scrollTo({ top: 0, behavior: 'instant' })
  }, [location])
  return <Switch>
    <Route path="/tests/:quizId/report">{(params) => <ReportRoute key={params.quizId} id={params.quizId} />}</Route>
    <Route path="/tests/:quizId">{(params) => <QuizRoute id={params.quizId} />}</Route>
    <Route path="/tests"><QuizHub /></Route>
    <Route><Missing /></Route>
  </Switch>
}
