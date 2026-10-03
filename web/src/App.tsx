import { useEffect } from 'react'
import { Redirect, Route, Switch, useLocation } from 'wouter'
import { TabBar } from './components/TabBar.tsx'
import { CatWindow, PrimaryButton, Screen } from './components/ui.tsx'
import { TERMS_VERSION } from './lib/consent.ts'
import { SessionProvider, useSession } from './lib/session.tsx'
import { track } from './lib/track.ts'
import CardPreview from './pages/CardPreview.tsx'
import ConsentScreen from './pages/ConsentScreen.tsx'
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

function usePageViews() {
  const [location] = useLocation()
  useEffect(() => {
    track('page_view', { path: location })
  }, [location])
}

function Gate() {
  const { status, error, profile, retry } = useSession()
  const [location] = useLocation()
  usePageViews()

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
  if (location === '/me') return <><Me key={profile.userId} /><TabBar /></>
  if (location === '/me/account') return <AccountProfile key={profile.userId} />
  if (profile.onboarded && profile.termsVersion !== TERMS_VERSION) return <ConsentScreen />
  if (!profile.onboarded && location !== '/hello') return <Redirect to="/hello" replace />

  return (
    <Switch key={profile.userId}>
      <Route path="/hello" component={Onboarding} />
      <Route path="/card" component={CardPreview} />
      <Route path="/games/:gameId"><GamePlayer key={profile.userId} /></Route>
      <Route path="/games"><GameHub /><TabBar /></Route>
      <Route path="/me/guardian" component={GuardianPage} />
      <Route path="/me/profile" component={ProfileEdit} />
      <Route path="/dex"><Dex /><TabBar /></Route>
      <Route path="/me"><Me /><TabBar /></Route>
      <Route path="/"><Home /><TabBar /></Route>
      <Route><Redirect to="/" replace /></Route>
    </Switch>
  )
}

export default function App() {
  const [location] = useLocation()
  if (location === '/admin' || location.startsWith('/admin/')) return <AdminPage />
  return (
    <SessionProvider>
      <Gate />
    </SessionProvider>
  )
}
