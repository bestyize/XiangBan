import type { Checkin } from '../../api/types.ts'
import { inApp, send } from '../../lib/bridge.ts'
import { useProfile, useSession } from '../../lib/session.tsx'
import { track } from '../../lib/track.ts'

/**
 * PRD 8.4: on day 7 the cat asks to come home (download 相伴 App). Inside the App there is no download
 * entry; a shortcut to the native chat takes its place.
 */
export function GoHome({ checkin }: { checkin: Checkin }) {
  const profile = useProfile()
  const { features } = useSession()

  if (inApp()) {
    return (
      <button type="button" onClick={() => send('open_native', { page: 'chat' })}
        className="mt-4 w-full rounded-2xl border-2 border-lamp/60 px-4 py-3 text-left text-window">
        去和{profile.companionName}聊两句
      </button>
    )
  }
  if (!checkin.goHome || !features.appDownloadUrl) return null

  return (
    <section className="mt-5 rounded-2xl bg-lamp/15 p-4" aria-label="带小伴回家">
      <p className="text-window">我们认识一周了，我想去你家看看。</p>
      <a
        href={features.appDownloadUrl}
        onClick={() => track('go_home_click', { streak: String(checkin.stats.streak) })}
        className="mt-3 block rounded-full bg-action px-5 py-3 text-center font-bold text-white"
      >
        带{profile.companionName}回家
      </a>
    </section>
  )
}
