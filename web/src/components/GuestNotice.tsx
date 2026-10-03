import { Link } from 'wouter'

export function GuestNotice({ className = '' }: { className?: string }) {
  return (
    <div className={`rounded-2xl border border-lamp/30 bg-lamp/10 p-4 text-xs leading-relaxed text-lamp ${className}`}>
      <div className="flex items-center gap-1.5 font-bold text-lamp">
        <span aria-hidden="true">🐾</span>
        <span>游客体验模式</span>
      </div>
      <p className="mt-1.5 text-window/80">
        游客数据仅保存在当前浏览器；清除浏览器数据、使用无痕模式或换设备后无法找回。创建账号后可保存并跨设备使用。
      </p>
      <div className="mt-3">
        <Link href="/login" className="inline-block rounded-full bg-lamp px-4 py-1.5 text-xs font-bold text-ink">
          登录 / 注册账号
        </Link>
      </div>
    </div>
  )
}
