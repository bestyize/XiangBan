import type { ComponentProps, ReactNode } from 'react'
import type { CatSpec } from '../cat/catSvg.ts'
import { Cat } from './Cat.tsx'

/** Includes `ref`, which React 19 passes to function components as a plain prop. */
type ButtonProps = ComponentProps<'button'>

export function PrimaryButton({ className = '', ...rest }: ButtonProps) {
  return (
    <button
      type="button"
      className={`w-full rounded-full bg-lamp px-6 py-3.5 text-base font-bold text-ink shadow-[0_6px_0_var(--color-lamp-deep)] transition active:translate-y-1 active:shadow-[0_2px_0_var(--color-lamp-deep)] disabled:opacity-50 ${className}`}
      {...rest}
    />
  )
}

export function QuietButton({ className = '', ...rest }: ButtonProps) {
  return (
    <button
      type="button"
      className={`rounded-full px-4 py-2 text-sm text-window/70 underline-offset-4 hover:text-window hover:underline disabled:opacity-50 ${className}`}
      {...rest}
    />
  )
}

/** The lit window at home where 小伴 waits; used wherever the cat speaks. */
interface CatWindowProps {
  spec: CatSpec
  label: string
  size?: number
  centered?: boolean
  className?: string
}

export function CatWindow({ spec, label, size = 208, centered = true, className = '' }: CatWindowProps) {
  const arch = size / 2
  return (
    <div className={`flex w-fit shrink-0 flex-col items-center ${centered ? 'mx-auto' : ''} ${className}`}>
      <div
        className="relative overflow-hidden bg-[radial-gradient(circle_at_50%_38%,#ffe9ad,#ffb938_62%,#e89a12)] shadow-[0_0_80px_rgba(255,185,56,0.32)]"
        style={{ width: size, height: size * 1.08, borderRadius: `${arch}px ${arch}px 14px 14px` }}
      >
        <div className="absolute inset-y-0 left-1/2 w-1.5 -translate-x-1/2 bg-dusk-deep/45" />
        <div className="absolute inset-x-0 top-[44%] h-1.5 bg-dusk-deep/45" />
        <Cat spec={spec} label={label} size={size * 0.86} className="absolute bottom-0 left-1/2 -translate-x-1/2" />
      </div>
      <div className="h-3 rounded-full bg-dusk-soft" style={{ width: size + 28 }} />
    </div>
  )
}

/** Speech bubble whose tail points up at the cat: centered under it, or under a cat on the left. */
export function Bubble({ children, align = 'center', className = '' }: { children: ReactNode; align?: 'center' | 'left'; className?: string }) {
  const layout = align === 'center' ? 'mx-auto max-w-[22rem] text-center before:left-1/2' : 'text-left before:left-16'
  return (
    <div
      className={`relative rounded-[22px] bg-window px-5 py-4 text-[17px] leading-relaxed text-ink before:absolute before:-top-2.5 before:-translate-x-1/2 before:border-x-[10px] before:border-b-[11px] before:border-x-transparent before:border-b-window before:content-[''] ${layout} ${className}`}
    >
      {children}
    </div>
  )
}

export function Screen({ children, className = '' }: { children: ReactNode; className?: string }) {
  return (
    <main
      className={`mx-auto flex min-h-dvh w-full max-w-[480px] flex-col px-5 ${className}`}
      style={{ paddingTop: 'calc(var(--safe-top) + 16px)', paddingBottom: 'calc(var(--safe-bottom) + 16px)' }}
    >
      {children}
    </main>
  )
}

export function FieldError({ message }: { message: string | null }) {
  if (!message) return null
  return (
    <p role="alert" className="mt-2 text-sm text-blush">
      {message}
    </p>
  )
}
