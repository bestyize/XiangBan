import { Link } from 'wouter'
import { DATA_OVERSEAS } from '../lib/consent.ts'

export interface ConsentState {
  terms: boolean
  overseas: boolean
}

export const consentComplete = (c: ConsentState) => c.terms && (!DATA_OVERSEAS || c.overseas)

/** Active opt-in checkboxes required by 《个人信息保护法》; never pre-ticked. */
export function ConsentChecks({ value, onChange }: { value: ConsentState; onChange: (v: ConsentState) => void }) {
  return (
    <div className="space-y-2 text-left text-[13px] leading-relaxed text-window/80">
      <label className="flex gap-2.5">
        <input type="checkbox" className="mt-0.5 size-4 shrink-0 accent-[var(--color-lamp)]" checked={value.terms}
          onChange={(e) => onChange({ ...value, terms: e.target.checked })} />
        <span>
          我已阅读并同意<Link href="/terms" className="text-lamp underline">《用户协议》</Link>和
          <Link href="/privacy" className="text-lamp underline">《隐私政策》</Link>
        </span>
      </label>
      {DATA_OVERSEAS && (
        <label className="flex gap-2.5">
          <input type="checkbox" className="mt-0.5 size-4 shrink-0 accent-[var(--color-lamp)]" checked={value.overseas}
            onChange={(e) => onChange({ ...value, overseas: e.target.checked })} />
          <span>内测期间服务器在境外，我单独同意把打卡记录等信息存放在境外服务器（正式上线前会迁回国内）</span>
        </label>
      )}
    </div>
  )
}
