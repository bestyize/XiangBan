import type { ComponentProps, CSSProperties, ReactNode } from 'react'
import { Robot, type RobotProps } from './Robot.tsx'

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

/** 小伴 at home under its own warm light; used wherever it speaks. */
interface RobotStageProps extends Omit<RobotProps, 'size'> {
  size?: number
  centered?: boolean
}

export function RobotStage({ size = 208, centered = true, className = '', ...robot }: RobotStageProps) {
  return (
    <div className={`relative flex w-fit shrink-0 flex-col items-center ${centered ? 'mx-auto' : ''} ${className}`}>
      <div aria-hidden="true" className="pointer-events-none absolute left-1/2 top-[42%] -translate-x-1/2 -translate-y-1/2 rounded-full bg-[radial-gradient(circle,rgba(255,196,110,0.32),rgba(255,196,110,0.08)_52%,transparent_70%)]"
        style={{ width: size * 1.45, height: size * 1.25 }} />
      <Robot {...robot} size={size} className="relative" />
    </div>
  )
}

/** Speech bubble whose tail points up at 小伴: centered under it, or under one on the left. */
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

export function Screen({ children, className = '', style }: { children: ReactNode; className?: string; style?: CSSProperties }) {
  return (
    <main
      className={`mx-auto flex min-h-dvh w-full max-w-[480px] flex-col px-5 ${className}`}
      style={{
        paddingTop: 'calc(var(--safe-top) + 16px)',
        paddingBottom: 'calc(var(--safe-bottom) + 16px)',
        ...style,
      }}
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
