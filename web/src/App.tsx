import { Redirect, Route, Switch } from 'wouter'
import { CatWindow, PrimaryButton, Screen } from './components/ui.tsx'
import { SessionProvider, useSession } from './lib/session.tsx'
import CardPreview from './pages/CardPreview.tsx'
import Home from './pages/Home.tsx'
import Onboarding from './pages/onboarding/Onboarding.tsx'

function Gate() {
  const { status, error, profile, retry } = useSession()

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

  return (
    <Switch>
      <Route path="/hello" component={Onboarding} />
      <Route path="/card" component={CardPreview} />
      <Route path="/">{profile.onboarded ? <Home /> : <Redirect to="/hello" replace />}</Route>
      <Route>
        <Redirect to="/" replace />
      </Route>
    </Switch>
  )
}

export default function App() {
  return (
    <SessionProvider>
      <Gate />
    </SessionProvider>
  )
}
