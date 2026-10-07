import { SubPage } from '../../components/SubPage.tsx'
import { useProfile } from '../../lib/session.tsx'
import { AfterworkSummary } from './AfterworkSummary.tsx'

export default function Habits() {
  const profile = useProfile()
  return (
    <SubPage title="下班习惯" back="/me">
      <div className="mt-2 rounded-2xl bg-dusk-soft/35 p-4">
        <AfterworkSummary timeZone={profile.timezone} />
      </div>
    </SubPage>
  )
}
