interface PunchButtonProps {
  onPress: () => void
  busy: boolean
}

/** The one big physical-feeling control in the app: a lit, chunky clock-out button. */
export function PunchButton({ onPress, busy }: PunchButtonProps) {
  return (
    <button
      type="button"
      onClick={onPress}
      disabled={busy}
      aria-busy={busy}
      className="relative mx-auto grid size-52 place-items-center rounded-full bg-[radial-gradient(circle_at_50%_30%,#ffd57a,#ffb938_58%,#f0a018)] text-ink shadow-[0_12px_0_var(--color-lamp-deep),0_0_0_12px_rgba(255,185,56,0.12),0_20px_60px_rgba(255,185,56,0.35)] transition-[transform,box-shadow] duration-100 active:translate-y-2.5 active:shadow-[0_2px_0_var(--color-lamp-deep),0_0_0_12px_rgba(255,185,56,0.12),0_10px_30px_rgba(255,185,56,0.3)] disabled:cursor-wait"
    >
      <span className={`font-display text-[42px] leading-none tracking-wide ${busy ? 'animate-pulse' : ''}`}>我下班了</span>
    </button>
  )
}
