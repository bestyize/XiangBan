import { Suspense, useEffect, useMemo, useState } from 'react'
import { Redirect, Route, Switch, useLocation } from 'wouter'
import Habits from './pages/me/Habits.tsx'
import Settings from './pages/me/Settings.tsx'
import { getToken } from './api/client.ts'
import { useAccountProgress } from './quizzes/useAccountProgress.ts'
import { KeepAlivePage } from './components/KeepAlivePage.tsx'
import { TabBar } from './components/TabBar.tsx'
import { RobotStage, PrimaryButton, Screen } from './components/ui.tsx'
import { SessionProvider, useSession } from './lib/session.tsx'
import { track } from './lib/track.ts'
import CardPreview from './pages/CardPreview.tsx'
import Home from './pages/Home.tsx'
import CheckinActivity from './pages/xiaban/CheckinActivity.tsx'
import LegalPage from './pages/LegalPage.tsx'
import GuardianPage from './pages/me/GuardianPage.tsx'
import Me from './pages/me/Me.tsx'
import ProfileEdit from './pages/me/ProfileEdit.tsx'
import Onboarding from './pages/onboarding/Onboarding.tsx'
import GameHub from './pages/games/GameHub.tsx'
import GamePlayer from './pages/games/GamePlayer.tsx'
import { LoginModal } from './components/LoginModal.tsx'
import { LoginModalContext, useLoginModal } from './lib/loginModal.ts'
import { readLoginReturn } from './lib/loginReturn.ts'
import { lazyPage } from './lib/lazyPage.ts'
import { ACTIVITY_PATH, rememberAfterOnboarding } from './lib/activity.ts'
import AdminPage from './pages/AdminPage.tsx'
import AccountProfile from './pages/me/AccountProfile.tsx'
import { opensGameDirectly, sharePageGameId } from './games/index.ts'
import { DisplaySettingsProvider } from './lib/displaySettings.tsx'

const Tests = lazyPage(() => import('./quizzes/Tests.tsx'))
const RobotLab = lazyPage(() => import('./pages/lab/RobotLab.tsx'))
const GameShare = lazyPage(() => import('./pages/games/GameShare.tsx'))
const QuizHub = lazyPage<{ active?: boolean }>(() => import('./quizzes/Tests.tsx').then((module) => ({ default: module.QuizHub })))

function AccountTabs({ location }: { location: string }) {
  const { profile } = useSession()
  return <>
    {profile?.onboarded && <>
      <KeepAlivePage active={location === '/'}><Home active={location === '/'} /></KeepAlivePage>
      <KeepAlivePage active={location === '/games'}><GameHub active={location === '/games'} /></KeepAlivePage>
    </>}
    <KeepAlivePage active={location === '/me'}><Me /></KeepAlivePage>
  </>
}

function usePageViews() {
  const [location] = useLocation()
  useEffect(() => {
    if (location !== '/tests' && !location.startsWith('/tests/')) track('page_view', { path: location })
  }, [location])
}

/** /login stays linkable (logout lands here): show the page it should sit over, with the dialog open. */
function LoginRoute() {
  const { isLoggedIn, profile } = useSession()
  const { openLogin } = useLoginModal()
  useEffect(() => { if (!isLoggedIn) openLogin() }, [isLoggedIn, openLogin])
  const background = isLoggedIn ? '/me' : readLoginReturn()?.path ?? (profile?.onboarded ? '/' : '/hello')
  return <Redirect to={background} replace />
}

function SessionRoutes() {
  const { status, error, profile, retry } = useSession()
  const [location] = useLocation()

  // Public tests never wait for an account or the onboarding gate.
  if (location === '/tests' || location.startsWith('/tests/') || sharePageGameId(location)) {
    return null
  }

  if (location === '/privacy') return <LegalPage doc="privacy" />
  if (location === '/terms') return <LegalPage doc="terms" />

  if (status === 'loading') {
    return (
      <Screen className="items-center justify-center">
        <RobotStage spec="waiting" label="小伴正在开门" size={170} />
        <p className="mt-6 text-window/70">小伴正在开门…</p>
      </Screen>
    )
  }
  if (status === 'error' || !profile) {
    return (
      <Screen className="items-center justify-center text-center">
        <RobotStage spec="offline" label="小伴没接到你" size={170} />
        <p className="mt-6 text-window/80">{error ?? '小伴没接到你。'}</p>
        <PrimaryButton className="mt-6 max-w-60" onClick={retry}>再试一次</PrimaryButton>
      </Screen>
    )
  }
  if (location === '/login') return <LoginRoute />
  if (opensGameDirectly(location, window.location.search)) return <GamePlayer key={profile.userId} />
  if (!profile.onboarded && !['/hello', '/me', '/me/account', '/me/settings'].includes(location)) {
    if (location === ACTIVITY_PATH) rememberAfterOnboarding(`${location}${window.location.search}`)
    return <Redirect to="/hello" replace />
  }

  return (
    <Switch key={profile.userId}>
      <Route path="/hello" component={Onboarding} />
      <Route path="/card" component={CardPreview} />
      <Route path={ACTIVITY_PATH} component={CheckinActivity} />
      <Route path="/chat"><Redirect to="/" replace /></Route>
      <Route path="/games/:gameId"><GamePlayer key={profile.userId} /></Route>
      <Route path="/games">{null}</Route>
      <Route path="/me/account"><AccountProfile /></Route>
      <Route path="/me/guardian" component={GuardianPage} />
      <Route path="/me/profile" component={ProfileEdit} />
      <Route path="/me/settings" component={Settings} />
      <Route path="/me/habits" component={Habits} />
      <Route path="/me">{null}</Route>
      <Route path="/">{null}</Route>
      <Route><Redirect to="/" replace /></Route>
    </Switch>
  )
}

function Gate() {
  const [location] = useLocation()
  const { status, profile } = useSession()
  return <>
    {status === 'ready' && profile && <AccountTabs key={profile.userId} location={location} />}
    <SessionRoutes />
  </>
}

function AppPages() {
  const [location] = useLocation()
  const { profile } = useSession()
  const tests = location === '/tests' || location.startsWith('/tests/')
  const quizSync = useAccountProgress(tests)
  const primary = ['/', '/games', '/tests', '/me'].includes(location)
  const shareGameId = sharePageGameId(location)
  usePageViews()
  useEffect(() => {
    const titles: Record<string, string> = { '/': '相伴', '/games': '玩一局｜相伴', '/tests': '趣味测试｜相伴', '/me': '我的｜相伴' }
    if (titles[location]) document.title = titles[location]
  }, [location])
  return <>
    <Gate />
    {shareGameId && <Suspense fallback={<Screen><p className="mt-8 text-window/60">正在打开战绩卡…</p></Screen>}><GameShare key={shareGameId} gameId={shareGameId} /></Suspense>}
    <KeepAlivePage active={location === '/tests'}>
      {quizSync.pending ? <Screen><p className="mt-8 text-window/60">正在找回你的测试进度…</p></Screen> : <>
        {quizSync.error && <p role="status" className="mx-auto max-w-[480px] px-5 pt-3 text-xs text-window/60">{quizSync.error}</p>}
        <Suspense fallback={<Screen><p className="mt-8 text-window/60">正在打开趣味测试…</p></Screen>}><QuizHub key={profile?.accountType === 'registered' ? profile.userId : 'guest'} active={location === '/tests'} /></Suspense>
      </>}
    </KeepAlivePage>
    {tests && location !== '/tests' && (quizSync.pending ? <Screen><p className="mt-8 text-window/60">正在找回你的测试进度…</p></Screen> : <><p role="status" className="mx-auto max-w-[480px] px-5 text-xs text-window/60">{quizSync.error}</p><Suspense fallback={<Screen><p className="mt-8 text-window/60">正在打开趣味测试…</p></Screen>}><Tests key={profile?.accountType === 'registered' ? profile.userId : 'guest'} /></Suspense></>)}
    {primary && (location === '/tests' || location === '/me' || profile?.onboarded) && <TabBar />}
  </>
}

export default function App() {
  const [location] = useLocation()
  const publicTests = location === '/tests' || location.startsWith('/tests/') || Boolean(sharePageGameId(location))
  const admin = location === '/admin' || location.startsWith('/admin/')
  const lab = location === '/lab/robot'
  const [sessionStarted, setSessionStarted] = useState((!publicTests && !admin) || Boolean(getToken()))
  const [loginOpen, setLoginOpen] = useState(false)
  // Public test pages run without a session; logging in from them has to start one.
  const login = useMemo(() => ({ openLogin: () => { setSessionStarted(true); setLoginOpen(true) } }), [])
  useEffect(() => { if (!publicTests && !admin) setSessionStarted(true) }, [publicTests, admin])
  // The phone-width desktop frame (index.css) is for the app only, not the admin console.
  useEffect(() => { document.documentElement.dataset.surface = admin || lab ? 'admin' : 'app' }, [admin, lab])
  if (admin) return <AdminPage />
  if (lab) return <Suspense fallback={null}><RobotLab /></Suspense>
  return <DisplaySettingsProvider><SessionProvider enabled={sessionStarted || !publicTests}>
    <LoginModalContext.Provider value={login}>
      <AppPages />
      <LoginModal open={loginOpen} onClose={() => setLoginOpen(false)} />
    </LoginModalContext.Provider>
  </SessionProvider></DisplaySettingsProvider>
}
