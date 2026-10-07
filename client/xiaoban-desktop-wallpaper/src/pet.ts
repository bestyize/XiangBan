// Desktop-owned character core, initially copied without changing art or motion.
import { Animator } from './companion/animator.ts'
import { Mind } from './companion/mind.ts'
import { toSpec, stillFrame, type RobotInput } from './companion/expressions.ts'
import { bodyTransform, earLight, earTransform, faceMarkup, faceOpacity, robotMarkup, shadowAttrs } from './companion/render.ts'
import type { Settings } from './bridge.ts'

export class PetView {
  readonly host: HTMLDivElement
  private animator = new Animator('idle', { reducedMotion: matchMedia('(prefers-reduced-motion: reduce)').matches })
  private mind = new Mind(this.animator, { autoSleep: true })
  private id = `pet${Math.random().toString(36).slice(2)}`
  private frameId = 0
  private last = 0
  private lookTimer = 0
  private config: Settings
  private lastExpression = ''
  private lastStatus = ''

  constructor(parent: HTMLElement, settings: Settings, private status?: (text: string) => void) {
    this.config = settings
    this.host = document.createElement('div')
    this.host.className = 'pet-stage'
    this.host.innerHTML = `<div class="pet-lamp"></div><div class="pet-art" role="img" aria-label="小伴" tabindex="0">${robotMarkup(stillFrame('idle'), this.id)}</div>`
    parent.append(this.host)
    this.update(settings)
    const art = this.host.querySelector('.pet-art')!
    const q = (selector: string) => this.host.querySelector<SVGElement>(selector)
    const body = q('[data-part="body"]'), face = q('[data-part="face"]'), shadow = q('[data-part="shadow"]')
    const ears = (['L', 'R'] as const).map(side => {
      const group = q(`[data-part="ear${side}"]`)
      return { group, halo: group?.querySelector('[data-part="halo"]'), light: group?.querySelector('[data-part="earLight"]') }
    })
    art.addEventListener('keydown', event => {
      if (event instanceof KeyboardEvent && ['Enter', ' '].includes(event.key)) { event.preventDefault(); this.poke() }
    })
    const tick = (now: number) => {
      this.frameId = requestAnimationFrame(tick)
      if (this.config.paused || document.hidden) { this.last = 0; return }
      if (this.last && now - this.last < 1000 / this.config.fps - 1) return
      const dt = this.last ? (now - this.last) / 1000 : 1 / 60
      this.last = now
      this.mind.tick(dt)
      const frame = this.animator.step(dt)
      body?.setAttribute('transform', bodyTransform(frame))
      if (face) { face.innerHTML = faceMarkup(frame, this.id); face.setAttribute('opacity', faceOpacity(frame).toFixed(2)) }
      const light = earLight(frame)
      ears.forEach(({ group, halo, light: inner }, i) => {
        group?.setAttribute('transform', earTransform(frame, i === 0 ? -1 : 1))
        halo?.setAttribute('opacity', light.toFixed(2))
        inner?.setAttribute('opacity', (0.25 + 0.75 * light).toFixed(2))
      })
      if (shadow) { const s = shadowAttrs(frame); shadow.setAttribute('rx', s.rx); shadow.setAttribute('opacity', s.opacity) }
      const current = toSpec(this.mind.current()).expression
      if (current !== this.lastStatus) { this.lastStatus = current; this.status?.(current) }
    }
    this.frameId = requestAnimationFrame(tick)
  }

  update(settings: Settings) {
    this.config = settings
    this.host.style.setProperty('--pet-size', `${settings.size}px`)
    if (settings.expression !== this.lastExpression) {
      this.lastExpression = settings.expression
      this.mind.setBase(settings.expression as RobotInput)
      this.mind.activity()
    }
  }

  look(x: number, y: number, moved = true) {
    if (!moved || this.config.paused) return
    const rect = this.host.querySelector('.pet-art')!.getBoundingClientRect()
    const reach = Math.max(rect.width, 240)
    this.animator.lookAt = { x: (x - (rect.left + rect.width / 2)) / reach, y: (y - (rect.top + rect.height * 0.55)) / reach }
    this.mind.activity()
    window.clearTimeout(this.lookTimer)
    this.lookTimer = window.setTimeout(() => { this.animator.lookAt = null }, 2500)
  }

  // Test actual transformed SVG fills. Glows/shadows stay click-through; the
  // breathing, ear movement and head tilt are included in the current CTMs.
  hit(x: number, y: number): boolean {
    const bounds = this.host.querySelector('.pet-art')!.getBoundingClientRect()
    if (x < bounds.left || x > bounds.right || y < bounds.top || y > bounds.bottom) return false
    return [...this.host.querySelectorAll<SVGGeometryElement>('svg path, svg rect, svg ellipse')].some(element => {
      if (element.closest('defs') || element.matches('[data-part="halo"], [data-part="shadow"]')) return false
      const fill = getComputedStyle(element).fill
      if (fill === 'none' || fill === 'transparent') return false
      const transform = element.getScreenCTM()
      return Boolean(transform && element.isPointInFill(new DOMPoint(x, y).matrixTransform(transform.inverse())))
    })
  }

  poke() { this.mind.poke() }
  destroy() { cancelAnimationFrame(this.frameId); clearTimeout(this.lookTimer); this.host.remove() }
}
