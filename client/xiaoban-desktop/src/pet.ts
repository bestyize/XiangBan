// Desktop-owned character core, initially copied without changing art or motion.
import { Animator } from './companion/animator.ts'
import { Mind, type Beat } from './companion/mind.ts'
import { Reactions } from './companion/reactions.ts'
import { toSpec, stillFrame, type ExpressionName, type RobotInput } from './companion/expressions.ts'
import { bodyTransform, earLight, earTransform, faceMarkup, faceOpacity, robotMarkup, shadowAttrs } from './companion/render.ts'
import { faceStyleOf, skinOf, type SkinId } from './companion/skins.ts'
import type { Settings } from './bridge.ts'
import type { PetDefinition } from './pets/model.ts'
import { petMarkup, petPalette, partTransform, poseFor } from './pets/render.ts'
import { hitImage, prepareImageMask } from './pets/images.ts'

export interface PetArtwork {
  render(seconds: number, expression: ExpressionName): void
  hit(x: number, y: number): boolean
  poke(): void
}

export class PetView {
  readonly host: HTMLDivElement
  private animator = new Animator('idle', { reducedMotion: matchMedia('(prefers-reduced-motion: reduce)').matches })
  private mind = new Mind(this.animator, { autoSleep: true })
  private id = `pet${Math.random().toString(36).slice(2)}`
  private frameId = 0
  private tick: FrameRequestCallback | null = null
  private last = 0
  private lastFaceMarkup = ''
  private lookTimer = 0
  private config: Settings
  private lastExpression = ''
  private lastStatus = ''
  private interactUntil = 0
  private skin: SkinId
  private partNodes = new Map<string, SVGElement>()
  private visiblePose?: SVGElement
  private artwork?: PetArtwork
  private parts!: { body: SVGElement | null; face: SVGElement | null; shadow: SVGElement | null; ears: { group: SVGElement | null; halo: Element | null | undefined; light: Element | null | undefined }[] }
  readonly reactions = new Reactions({
    play: beats => this.mind.play(beats),
    stop: () => this.mind.play([]),
    lookAt: target => { this.animator.lookAt = target },
    busy: () => this.busy(),
  })
  private careExpression = (event: Event) => {
    if (event instanceof CustomEvent && event.detail === 'happy') this.temporaryExpression('happy')
  }

  constructor(parent: HTMLElement, settings: Settings, private status?: (text: string) => void, private definition?: PetDefinition) {
    this.config = settings
    this.skin = skinOf(settings.companionSkin).id
    this.host = document.createElement('div')
    this.host.className = 'pet-stage'
    this.host.innerHTML = `<div class="pet-art" role="img" aria-label="小伴" tabindex="0">${this.markup()}</div>`
    parent.append(this.host)
    window.addEventListener('care-expression', this.careExpression)
    this.bind()
    this.update(settings)
    const art = this.host.querySelector('.pet-art')!
    art.addEventListener('keydown', event => {
      if (event instanceof KeyboardEvent && ['Enter', ' '].includes(event.key)) { event.preventDefault(); this.poke() }
    })
    const tick = (now: number) => {
      this.frameId = 0
      if (this.config.paused || document.hidden) { this.last = 0; return }
      this.schedule()
      if (this.last && now - this.last < 1000 / this.config.fps - 1) return
      const dt = this.last ? (now - this.last) / 1000 : 1 / 60
      this.last = now
      this.reactions.tick(dt)
      this.mind.tick(dt)
      const current = toSpec(this.mind.current()).expression
      if (current !== this.lastStatus) { this.lastStatus = current; this.status?.(current) }
      if (this.artwork) { this.artwork.render(dt, current); return }
      const frame = this.animator.step(dt)
      const { body, face, shadow, ears } = this.parts
      body?.setAttribute('transform', bodyTransform(frame))
      if (face) {
        const markup = faceMarkup(frame, this.id, this.definition?petPalette(this.definition):faceStyleOf(skinOf(this.skin)).palette)
        if (markup !== this.lastFaceMarkup) { face.innerHTML = markup; this.lastFaceMarkup = markup }
        face.setAttribute('opacity', faceOpacity(frame).toFixed(2))
      }
      if (this.definition) {
        if (this.definition.poses) this.showPose(poseFor(this.definition.poses, current, frame))
        for (const part of this.definition.parts) {
          const node = this.partNodes.get(part.id)
          if (!this.visiblePose || node === this.visiblePose) node?.setAttribute('transform', partTransform(part, frame))
        }
      }
      const light = earLight(frame)
      ears.forEach(({ group, halo, light: inner }, i) => {
        group?.setAttribute('transform', earTransform(frame, i === 0 ? -1 : 1, skinOf(this.skin).earPivot))
        halo?.setAttribute('opacity', light.toFixed(2))
        inner?.setAttribute('opacity', (0.25 + 0.75 * light).toFixed(2))
      })
      if (shadow) { const s = shadowAttrs(frame); shadow.setAttribute('rx', s.rx); shadow.setAttribute('opacity', s.opacity) }
    }
    this.tick = tick
    document.addEventListener('visibilitychange', this.onVisibility)
    this.schedule()
  }

  private schedule() {
    if (!this.frameId && this.tick && !this.config.paused && !document.hidden) this.frameId = requestAnimationFrame(this.tick)
  }
  private onVisibility = () => {
    if (document.hidden) { cancelAnimationFrame(this.frameId); this.frameId = 0; this.last = 0 }
    else this.schedule()
  }

  private bind() {
    this.lastFaceMarkup = ''
    const q = (selector: string) => this.host.querySelector<SVGElement>(selector)
    const ears = (['L', 'R'] as const).map(side => {
      const group = q(`[data-part="ear${side}"]`)
      return { group, halo: group?.querySelector('[data-part="halo"]'), light: group?.querySelector('[data-part="earLight"]') }
    })
    this.parts = { body: q('[data-part="body"]'), face: q('[data-part="face"]'), shadow: q('[data-part="shadow"]'), ears }
    this.partNodes.clear()
    for (const part of this.host.querySelectorAll<SVGElement>('[data-pet-part]')) this.partNodes.set(part.dataset.petPart!, part)
    this.visiblePose = undefined
    const initialPose = this.definition?.poses && q('[data-pet-part]:not([display="none"])')?.dataset.petPart
    if (initialPose) this.showPose(initialPose)
    this.host.querySelector('.pet-art')?.setAttribute('aria-label', this.definition?.name ?? '小伴')
  }
  private showPose(id: string) {
    const node = this.partNodes.get(id)
    if (!node || this.visiblePose === node) return
    this.visiblePose?.setAttribute('display', 'none')
    node.removeAttribute('display')
    this.visiblePose = node
    // A click can be followed immediately by a drag while the reaction is on screen.
    node.querySelectorAll<SVGImageElement>('image[data-pet-image]').forEach(prepareImageMask)
  }
  private markup() {
    const expression = this.config.expression as ExpressionName
    return this.definition ? petMarkup(this.definition, stillFrame(expression), this.id, expression, true)
      : robotMarkup(stillFrame('idle'), this.id, undefined, this.skin)
  }
  setDefinition(definition:PetDefinition){if(definition===this.definition||JSON.stringify(definition)===JSON.stringify(this.definition))return;this.definition=definition;this.host.querySelector('.pet-art')!.innerHTML=this.markup();this.bind()}

  setArtwork(artwork?: PetArtwork) {
    this.artwork = artwork
    this.redrawArtwork()
  }
  redrawArtwork() { this.artwork?.render(0, toSpec(this.mind.current()).expression) }

  update(settings: Settings) {
    this.config = settings
    if (settings.paused) { cancelAnimationFrame(this.frameId); this.frameId = 0; this.last = 0 }
    else this.schedule()
    const skin = skinOf(settings.companionSkin).id
    if (skin !== this.skin && !this.definition) {
      this.skin = skin
      this.host.querySelector('.pet-art')!.innerHTML = robotMarkup(stillFrame('idle'), this.id, undefined, skin)
      this.bind()
    }
    this.host.style.setProperty('--pet-size', `${settings.size}px`)
    if(this.definition)this.host.style.height=`${settings.size}px`
    if (settings.expression !== this.lastExpression) {
      this.lastExpression = settings.expression
      this.mind.play([])
      this.mind.setBase(settings.expression as RobotInput)
      this.mind.activity()
      if (this.definition?.poses) this.showPose(this.definition.poses[settings.expression as ExpressionName] ?? this.definition.poses.idle)
    }
    this.redrawArtwork()
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
    if (this.artwork) return this.artwork.hit(x, y)
    const bounds = this.host.querySelector('.pet-art')!.getBoundingClientRect()
    if (x < bounds.left || x > bounds.right || y < bounds.top || y > bounds.bottom) return false
    const artwork = this.visiblePose ?? this.host
    return [...artwork.querySelectorAll<SVGImageElement>('image[data-pet-image]')].some(element => hitImage(element,x,y)) || [...artwork.querySelectorAll<SVGGeometryElement>('path, rect, ellipse, circle, polygon')].some(element => {
      if (element.closest('defs') || element.matches('[data-part="halo"], [data-part="shadow"], [data-part="aura"]')) return false
      const fill = getComputedStyle(element).fill
      if (fill === 'none' || fill === 'transparent') return false
      const transform = element.getScreenCTM()
      return Boolean(transform && element.isPointInFill(new DOMPoint(x, y).matrixTransform(transform.inverse())))
    })
  }

  /** Direct handling (click, drag) or a care expression: effect reactions stay out of the way. */
  interact(seconds = 4) { this.interactUntil = Math.max(this.interactUntil, performance.now() + seconds * 1000) }
  private busy() { return this.config.paused || this.mind.dozing || performance.now() < this.interactUntil }

  playBeats(beats: Beat[]) {
    if (this.config.paused || this.mind.dozing) return
    this.mind.activity()
    this.interact(beats.reduce((sum, beat) => sum + beat.hold, 0))
    this.mind.play(beats)
  }

  poke() {
    if (this.artwork && !this.config.paused) this.artwork.poke()
    this.interact()
    if (!this.mind.dozing && this.reactions.wish()) { this.mind.activity(); this.animator.twitchEar(0, 0.8); return }
    this.mind.poke()
  }
  temporaryExpression(expression: RobotInput, seconds = 10) {
    if (this.config.paused) return
    this.mind.activity()
    this.interact(Math.min(seconds, 10))
    this.mind.play([{ expr: expression, hold: Math.max(0.5, Math.min(seconds, 10)) }])
  }
  destroy() { this.reactions.destroy(); cancelAnimationFrame(this.frameId); this.tick = null; clearTimeout(this.lookTimer); document.removeEventListener('visibilitychange', this.onVisibility); window.removeEventListener('care-expression', this.careExpression); this.host.remove() }
}

