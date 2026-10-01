import { Bubble, CatWindow, Screen } from '../components/ui.tsx'
import { useProfile } from '../lib/session.tsx'

export default function Home() {
  const profile = useProfile()
  const name = profile.nickname ?? '你'
  return (
    <Screen className="items-center justify-center">
      <h1 className="font-display text-5xl text-lamp">下班了么</h1>
      <CatWindow className="mt-8" spec={{ eyes: 'open', mouth: 'cat', blush: true }} label={`${profile.companionName}在窗台上等你`} />
      <Bubble className="mt-5">{name}好，我是{profile.companionName}。下班的时候来找我。</Bubble>
    </Screen>
  )
}
