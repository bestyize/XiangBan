import { Link } from 'wouter'
import { useLoginModal } from '../lib/loginModal.ts'
import { useProfile, useSession } from '../lib/session.tsx'

export function AccountBadge({ compact = false }: { compact?: boolean }) {
  const profile = useProfile()
  const { isLoggedIn } = useSession()
  const { openLogin } = useLoginModal()
  const className = `flex min-h-11 shrink-0 items-center gap-1.5 rounded-full border border-lamp/40 py-1.5 text-lamp ${compact ? 'px-2 text-[11px] min-[400px]:px-3 min-[400px]:text-xs' : 'px-3 text-xs'}`
  if (!isLoggedIn) return <button type="button" onClick={openLogin} className={className}><span className={compact ? 'max-w-14 truncate min-[400px]:max-w-20' : 'max-w-24 truncate'}>登录 / 注册</span></button>
  return <Link href="/me" className={className}>
    {isLoggedIn && profile.avatarUrl && <img src={profile.avatarUrl} alt="" className={compact ? 'hidden size-4 rounded-full object-cover min-[400px]:block' : 'size-5 rounded-full object-cover'} />}
    <span className={compact ? 'max-w-14 truncate min-[400px]:max-w-20' : 'max-w-24 truncate'}>{isLoggedIn ? profile.nickname ?? `用户 #${profile.userId}` : '登录 / 注册'}</span>
  </Link>
}
