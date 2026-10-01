import { useState } from 'react'
import { api } from '../api/client.ts'
import type { Profile } from '../api/types.ts'
import { ConsentChecks, consentComplete, type ConsentState } from '../components/ConsentChecks.tsx'
import { Bubble, CatWindow, FieldError, PrimaryButton, Screen } from '../components/ui.tsx'
import { TERMS_VERSION } from '../lib/consent.ts'
import { useSession } from '../lib/session.tsx'

/** Shown to existing users when the terms change, before anything else. */
export default function ConsentScreen() {
  const { setProfile } = useSession()
  const [consent, setConsent] = useState<ConsentState>({ terms: false, overseas: false })
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function accept() {
    setBusy(true)
    setError(null)
    try {
      setProfile(await api<Profile>('/api/consent', { method: 'PUT', body: { termsVersion: TERMS_VERSION, overseas: consent.overseas } }))
    } catch (e) {
      setError((e as Error).message)
      setBusy(false)
    }
  }

  return (
    <Screen className="justify-center">
      <CatWindow spec={{ eyes: 'open', mouth: 'smile', blush: true }} label="小伴" size={150} />
      <Bubble className="mt-5">《用户协议》和《隐私政策》更新了，看一眼再继续吧。</Bubble>
      <div className="mt-6">
        <ConsentChecks value={consent} onChange={setConsent} />
        <FieldError message={error} />
      </div>
      <PrimaryButton className="mt-6" disabled={busy || !consentComplete(consent)} onClick={accept}>同意并继续</PrimaryButton>
    </Screen>
  )
}
