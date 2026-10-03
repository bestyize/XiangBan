import { Link } from 'wouter'
import { useProfile, useSession } from '../lib/session.tsx'

export function AccountBadge() {
  const profile = useProfile()
  const { isLoggedIn } = useSession()
  return <Link href={isLoggedIn ? '/me' : '/login'} className="flex items-center gap-1.5 rounded-full border border-lamp/40 px-3 py-1.5 text-xs text-lamp">
    {isLoggedIn && profile.avatarUrl && <img src={profile.avatarUrl} alt="" className="size-5 rounded-full object-cover" />}
    <span className="max-w-24 truncate">{isLoggedIn ? profile.nickname ?? `用户 #${profile.userId}` : '登录 / 注册'}</span>
  </Link>
}
