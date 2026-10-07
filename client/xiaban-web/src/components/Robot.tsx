import { useEffect, useId, useImperativeHandle, useMemo, useRef, type Ref } from 'react'
import { Animator } from '../robot/animator.ts'
import { stillFrame, toSpec, type RobotInput } from '../robot/expressions.ts'
import { Mind, type Beat } from '../robot/mind.ts'
import {
  bodyTransform, earLight, earTransform, faceMarkup, faceOpacity, robotMarkup, shadowAttrs,
} from '../robot/render.ts'

export interface RobotHandle {
  /** Play a short sequence of expressions, then fall back to the current state. */
  play: (beats: Beat[]) => void
  poke: () => void
  /** Wakes the robot / resets its doze timer. */
  activity: () => void
}

export interface RobotProps {
  spec?: RobotInput
  label: string
  size?: number
  className?: string
  /** Animate with springs and the life layer. false renders a single still frame (thumbnails, many robots). */
  alive?: boolean
  /** Tap to poke, eyes follow the pointer. */
  interactive?: boolean
  /** true for synthetic lip movement, or a live voice amplitude 0..1. */
  speaking?: boolean | number
  thinking?: boolean
  listening?: boolean
  autoSleep?: boolean
  /** Frame-rate cap, for pages that show many live robots. */
  fps?: number
  ref?: Ref<RobotHandle>
}

type Tick = (now: number) => void
const ticks = new Set<Tick>()
let frameId = 0
function loop(now: number) {
  for (const tick of ticks) tick(now)
  frameId = ticks.size ? requestAnimationFrame(loop) : 0
}
function subscribe(tick: Tick) {
  ticks.add(tick)
  if (!frameId) frameId = requestAnimationFrame(loop)
  return () => { ticks.delete(tick) }
}

const reducedMotion = () => typeof matchMedia === 'function' && matchMedia('(prefers-reduced-motion: reduce)').matches

export function Robot({
  spec = 'idle', label, size = 200, className = '', alive = true, interactive = false,
  speaking = false, thinking = false, listening = false, autoSleep = false, fps = 60, ref,
}: RobotProps) {
  const id = `rb${useId().replace(/[^a-zA-Z0-9]/g, '')}`
  const host = useRef<HTMLSpanElement>(null)
  const brain = useRef<{ animator: Animator; mind: Mind } | null>(null)
  const specKey = JSON.stringify(toSpec(spec))

  // Live robots keep their DOM and get updated in place; the markup is only built once per instance.
  // Still robots re-render whenever the spec changes. The markup comes from our own constants, never user input.
  const stillKey = alive ? '' : specKey
  // eslint-disable-next-line react-hooks/exhaustive-deps
  const html = useMemo(() => robotMarkup(stillFrame(spec), id), [id, stillKey])

  useImperativeHandle(ref, () => ({
    play: (beats) => brain.current?.mind.play(beats),
    poke: () => brain.current?.mind.poke(),
    activity: () => brain.current?.mind.activity(),
  }), [])

  useEffect(() => {
    if (!alive || !host.current) return
    const root = host.current
    const animator = new Animator(spec, { reducedMotion: reducedMotion() })
    const mind = new Mind(animator, { autoSleep })
    mind.setBase(spec)
    brain.current = { animator, mind }

    const q = (sel: string) => root.querySelector(sel) as SVGElement | null
    const body = q('[data-part="body"]')
    const face = q('[data-part="face"]')
    const shadow = q('[data-part="shadow"]')
    const ears = (['L', 'R'] as const).map((s) => {
      const group = q(`[data-part="ear${s}"]`)
      return { group, halo: group?.querySelector('[data-part="halo"]'), light: group?.querySelector('[data-part="earLight"]') }
    })

    let visible = true
    const observer = typeof IntersectionObserver === 'function'
      ? new IntersectionObserver(([entry]) => { visible = entry.isIntersecting })
      : null
    observer?.observe(root)

    let last = 0
    const minGap = 1000 / Math.max(1, fps) - 1
    const unsubscribe = subscribe((now) => {
      if (!visible || document.hidden) {
        last = 0
        return
      }
      if (last && now - last < minGap) return
      const dt = last ? (now - last) / 1000 : 1 / 60
      last = now
      mind.tick(dt)
      const frame = animator.step(dt)
      body?.setAttribute('transform', bodyTransform(frame))
      if (face) {
        face.innerHTML = faceMarkup(frame, id)
        face.setAttribute('opacity', faceOpacity(frame).toFixed(2))
      }
      const light = earLight(frame)
      ears.forEach(({ group, halo, light: inner }, i) => {
        group?.setAttribute('transform', earTransform(frame, i === 0 ? -1 : 1))
        halo?.setAttribute('opacity', light.toFixed(2))
        inner?.setAttribute('opacity', (0.25 + 0.75 * light).toFixed(2))
      })
      if (shadow) {
        const s = shadowAttrs(frame)
        shadow.setAttribute('rx', s.rx)
        shadow.setAttribute('opacity', s.opacity)
      }
    })

    return () => {
      unsubscribe()
      observer?.disconnect()
      brain.current = null
    }
    // The brain lives for the lifetime of the element; later prop changes are fed in by the effects below.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [alive, id, autoSleep, fps])

  useEffect(() => {
    brain.current?.mind.setBase(spec)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [specKey, alive])

  useEffect(() => {
    const b = brain.current
    if (!b) return
    b.mind.setFlags({ speaking: speaking !== false && speaking !== 0, thinking, listening })
    b.animator.speaking = typeof speaking === 'number' ? speaking : speaking ? 1 : 0
  }, [speaking, thinking, listening, alive])

  useEffect(() => {
    if (!alive || !interactive) return
    let idleTimer = 0
    const onMove = (event: PointerEvent) => {
      const b = brain.current
      const el = host.current
      if (!b || !el) return
      const rect = el.getBoundingClientRect()
      const reach = Math.max(rect.width, 240)
      b.animator.lookAt = {
        x: (event.clientX - (rect.left + rect.width / 2)) / reach,
        y: (event.clientY - (rect.top + rect.height * 0.55)) / reach,
      }
      b.mind.activity()
      window.clearTimeout(idleTimer)
      idleTimer = window.setTimeout(() => { if (brain.current) brain.current.animator.lookAt = null }, 2500)
    }
    window.addEventListener('pointermove', onMove, { passive: true })
    window.addEventListener('pointerdown', onMove, { passive: true })
    return () => {
      window.removeEventListener('pointermove', onMove)
      window.removeEventListener('pointerdown', onMove)
      window.clearTimeout(idleTimer)
    }
  }, [alive, interactive])

  return (
    <span
      ref={host}
      role="img"
      aria-label={label}
      className={`inline-block shrink-0 select-none ${interactive ? 'cursor-pointer touch-manipulation' : ''} ${className}`}
      style={{ width: size, height: size * 0.875 }}
      onPointerDown={interactive ? () => brain.current?.mind.poke() : undefined}
      dangerouslySetInnerHTML={{ __html: html }}
    />
  )
}
