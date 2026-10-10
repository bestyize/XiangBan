import type { ReactNode } from 'react'
import type { RobotInput } from '../robot/expressions.ts'
import { Robot } from './Robot.tsx'

/** 小伴 beside a short line of its own: how every tab greets you, so it feels like doing things together. */
export function CompanionNote({ spec, name, children, className = '' }: { spec: RobotInput; name: string; children: ReactNode; className?: string }) {
  return (
    <div className={`flex items-center gap-2 ${className}`}>
      <Robot spec={spec} label={name} size={76} interactive fps={30} />
      <p className="relative min-w-0 flex-1 rounded-2xl bg-surface px-3.5 py-2.5 text-left text-[14px] leading-relaxed text-ink before:absolute before:-left-2 before:top-1/2 before:-translate-y-1/2 before:border-y-[7px] before:border-r-[9px] before:border-y-transparent before:border-r-surface before:content-['']">
        {children}
      </p>
    </div>
  )
}
