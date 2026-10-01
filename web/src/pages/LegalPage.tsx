import { privacyPolicy, userAgreement } from '../content/legal.ts'
import { QuietButton, Screen } from '../components/ui.tsx'

export default function LegalPage({ doc }: { doc: 'privacy' | 'terms' }) {
  const content = doc === 'privacy' ? privacyPolicy : userAgreement
  return (
    <Screen>
      <header className="flex h-10 items-center">
        <QuietButton className="px-0!" onClick={() => history.back()}>返回</QuietButton>
      </header>
      <article className="pb-10 text-[15px] leading-[1.85] text-window/85">
        <h1 className="mt-2 text-2xl font-bold text-window">{content.title}</h1>
        <p className="mt-1 text-xs text-window/50">版本 {content.version}，更新于 {content.updatedAt}</p>
        {content.intro.map((p) => <p key={p} className="mt-4">{p}</p>)}
        {content.sections.map((s) => (
          <section key={s.heading} className="mt-7">
            <h2 className="text-base font-bold text-window">{s.heading}</h2>
            {s.paragraphs.map((p) => <p key={p} className="mt-2">{p}</p>)}
          </section>
        ))}
      </article>
    </Screen>
  )
}
