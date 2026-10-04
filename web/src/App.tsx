import { lazy, Suspense, useEffect, useState } from 'react'
import { Redirect, Route, Switch, useLocation } from 'wouter'
import { getToken } from './api/client.ts'
import { useAccountProgress } from './quizzes/useAccountProgress.ts'
import { KeepAlivePage } from './components/KeepAlivePage.tsx'
import { TabBar } from './components/TabBar.tsx'
import { CatWindow, PrimaryButton, Screen } from './components/ui.tsx'
import { SessionProvider, useSession } from './lib/session.tsx'
import { track } from './lib/track.ts'
import CardPreview from './pages/CardPreview.tsx'
import Dex from './pages/Dex.tsx'
import Home from './pages/Home.tsx'
import LegalPage from './pages/LegalPage.tsx'
import GuardianPage from './pages/me/GuardianPage.tsx'
import Me from './pages/me/Me.tsx'
import ProfileEdit from './pages/me/ProfileEdit.tsx'
import Onboarding from './pages/onboarding/Onboarding.tsx'
import GameHub from './pages/games/GameHub.tsx'
import GamePlayer from './pages/games/GamePlayer.tsx'
import EmailLogin from './pages/EmailLogin.tsx'
import AdminPage from './pages/AdminPage.tsx'
import AccountProfile from './pages/me/AccountProfile.tsx'

const Tests = lazy(() => import('./quizzes/Tests.tsx'))
const QuizHub = lazy(() => import('./quizzes/Tests.tsx').then((module) => ({ default: module.QuizHub })))

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

function SessionRoutes() {
  const { status, error, profile, retry } = useSession()
  const [location] = useLocation()

  // Public tests never wait for an account or the onboarding gate.
  if (location === '/tests' || location.startsWith('/tests/')) {
    return null
  }

  if (location === '/privacy') return <LegalPage doc="privacy" />
  if (location === '/terms') return <LegalPage doc="terms" />

  if (status === 'loading') {
    return (
      <Screen className="items-center justify-center">
        <CatWindow spec={{ eyes: 'sleepy', mouth: 'cat' }} label="小伴正在开门" size={160} />
        <p className="mt-6 text-window/70">小伴正在开门…</p>
      </Screen>
    )
  }
  if (status === 'error' || !profile) {
    return (
      <Screen className="items-center justify-center text-center">
        <CatWindow spec={{ eyes: 'worried', mouth: 'wavy' }} label="小伴没接到你" size={160} />
        <p className="mt-6 text-window/80">{error ?? '小伴没接到你。'}</p>
        <PrimaryButton className="mt-6 max-w-60" onClick={retry}>再试一次</PrimaryButton>
      </Screen>
    )
  }
  if (location === '/login') return <EmailLogin />
  if (!profile.onboarded && !['/hello', '/me', '/me/account'].includes(location)) return <Redirect to="/hello" replace />

  return (
    <Switch key={profile.userId}>
      <Route path="/hello" component={Onboarding} />
      <Route path="/card" component={CardPreview} />
      <Route path="/games/:gameId"><GamePlayer key={profile.userId} /></Route>
      <Route path="/games">{null}</Route>
      <Route path="/me/account"><AccountProfile /></Route>
      <Route path="/me/guardian" component={GuardianPage} />
      <Route path="/me/profile" component={ProfileEdit} />
      <Route path="/dex"><Dex /></Route>
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
  const primary = ['/', '/games', '/tests', '/me', '/dex'].includes(location)
  usePageViews()
  useEffect(() => {
    const titles: Record<string, string> = { '/': '下班了么', '/games': '玩一局', '/tests': '趣味测试', '/me': '我的', '/dex': '表情图鉴' }
    if (titles[location]) document.title = `${titles[location]}｜下班了么`
  }, [location])
  return <>
    <Gate />
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
  const publicTests = location === '/tests' || location.startsWith('/tests/')
  const admin = location === '/admin' || location.startsWith('/admin/')
  const [sessionStarted, setSessionStarted] = useState((!publicTests && !admin) || Boolean(getToken()))
  useEffect(() => { if (!publicTests && !admin) setSessionStarted(true) }, [publicTests, admin])
  if (admin) return <AdminPage />
  return <SessionProvider enabled={sessionStarted || !publicTests}><AppPages /></SessionProvider>
}
