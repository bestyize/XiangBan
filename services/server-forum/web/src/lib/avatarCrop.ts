// 更换头像的圆形裁剪：纯计算部分无依赖，可在 Node 里测试；画布交互在 AvatarCropper。
// 设计稿：client/xiaoban-multiplatform/docs/design-specs/prototypes/avatar-editor.html

/** 导出头像的边长（像素）。 */
export const AVATAR_EXPORT_SIZE = 512
/** 本机图片的大小上限。 */
export const AVATAR_MAX_BYTES = 10 * 1024 * 1024
/** 圆内对应原图短边少于这么多像素时提示可能模糊。 */
export const AVATAR_SMALL_PX = 256
export const AVATAR_MAX_ZOOM = 4

/** 圆心在画面中心；tx/ty 是图片中心相对圆心的偏移（画面像素）。 */
export interface CropState {
  iw: number
  ih: number
  zoom: number
  tx: number
  ty: number
}

export function initialCrop(iw: number, ih: number): CropState {
  return { iw, ih, zoom: 1, tx: 0, ty: 0 }
}

/** zoom = 1 时图片短边正好等于圆的直径。 */
export function scaleOf(state: CropState, diameter: number): number {
  return (diameter / Math.min(state.iw, state.ih)) * state.zoom
}

/** 让图片始终盖满圆：偏移不超过图片多出圆的一半。 */
export function clampCrop(state: CropState, diameter: number): CropState {
  const s = scaleOf(state, diameter)
  const mx = Math.max(0, (state.iw * s - diameter) / 2)
  const my = Math.max(0, (state.ih * s - diameter) / 2)
  return { ...state, tx: Math.min(mx, Math.max(-mx, state.tx)), ty: Math.min(my, Math.max(-my, state.ty)) }
}

export function panCrop(state: CropState, diameter: number, dx: number, dy: number): CropState {
  return clampCrop({ ...state, tx: state.tx + dx, ty: state.ty + dy }, diameter)
}

/** 以 (px, py)（相对圆心）为不动点缩放到 zoom。 */
export function zoomCrop(state: CropState, diameter: number, zoom: number, px = 0, py = 0): CropState {
  const z = Math.min(AVATAR_MAX_ZOOM, Math.max(1, zoom))
  const k = z / state.zoom
  return clampCrop({ ...state, zoom: z, tx: px - (px - state.tx) * k, ty: py - (py - state.ty) * k }, diameter)
}

/** 原图顺时针旋转 90° 后的状态（宽高互换，偏移跟着转）。 */
export function rotateCrop(state: CropState, diameter: number): CropState {
  return clampCrop({ iw: state.ih, ih: state.iw, zoom: state.zoom, tx: -state.ty, ty: state.tx }, diameter)
}

/** 圆的外接正方形在原图里的位置，用于导出和小图预览。 */
export function sourceRect(state: CropState, diameter: number): { sx: number; sy: number; size: number } {
  const s = scaleOf(state, diameter)
  return { sx: state.iw / 2 - (diameter / 2 + state.tx) / s, sy: state.ih / 2 - (diameter / 2 + state.ty) / s, size: diameter / s }
}

export function isSmallCrop(state: CropState): boolean {
  return Math.min(state.iw, state.ih) / state.zoom < AVATAR_SMALL_PX
}

/** 本机文件的检查；返回给用户看的原因，通过时返回 null。 */
export function checkAvatarFile(file: { type: string; size: number }): string | null {
  if (!/^image\//i.test(file.type)) return '请选择图片文件'
  if (file.size > AVATAR_MAX_BYTES) return '图片超过 10 MB，换一张试试'
  if (file.size <= 0) return '这张图片是空的，换一张试试'
  return null
}

/** 双击：放大到 2.2 倍，已放大时还原。 */
export function doubleTapZoom(zoom: number): number {
  return zoom > 1.4 ? 1 : 2.2
}

// ---------- 画布 ----------

type Source = CanvasImageSource & { width: number; height: number }

/** 把本机图片读成画布，只在浏览器内处理，不上传原图。 */
export async function loadAvatarSource(file: File): Promise<HTMLCanvasElement> {
  const url = URL.createObjectURL(file)
  try {
    const img = new Image()
    img.decoding = 'async'
    img.src = url
    await img.decode()
    const canvas = document.createElement('canvas')
    canvas.width = img.naturalWidth
    canvas.height = img.naturalHeight
    canvas.getContext('2d')!.drawImage(img, 0, 0)
    return canvas
  } catch {
    throw new Error('这张图片打不开，换一张试试')
  } finally {
    URL.revokeObjectURL(url)
  }
}

export function rotateSource(src: Source): HTMLCanvasElement {
  const c = document.createElement('canvas')
  c.width = src.height
  c.height = src.width
  const ctx = c.getContext('2d')!
  ctx.translate(src.height, 0)
  ctx.rotate(Math.PI / 2)
  ctx.drawImage(src, 0, 0)
  return c
}

/** 把圆形区域画到 size×size 的画布上（白底，方图；显示时由 CSS 裁成圆）。 */
export function paintCrop(ctx: CanvasRenderingContext2D, size: number, src: Source | null, state: CropState | null, diameter: number) {
  ctx.setTransform(1, 0, 0, 1, 0, 0)
  ctx.fillStyle = '#fff'
  ctx.fillRect(0, 0, size, size)
  if (!src || !state) return
  const r = sourceRect(state, diameter)
  ctx.imageSmoothingEnabled = true
  ctx.imageSmoothingQuality = 'high'
  ctx.drawImage(src, r.sx, r.sy, r.size, r.size, 0, 0, size, size)
}

function canvasToBlob(canvas: HTMLCanvasElement, type: string, quality: number): Promise<Blob | null> {
  return new Promise(resolve => canvas.toBlob(resolve, type, quality))
}

export async function exportAvatar(src: Source, state: CropState, diameter: number): Promise<File> {
  const canvas = document.createElement('canvas')
  canvas.width = canvas.height = AVATAR_EXPORT_SIZE
  paintCrop(canvas.getContext('2d')!, AVATAR_EXPORT_SIZE, src, state, diameter)
  const webp = await canvasToBlob(canvas, 'image/webp', 0.88)
  if (webp && webp.type === 'image/webp') return new File([webp], 'avatar.webp', { type: 'image/webp' })
  const jpeg = await canvasToBlob(canvas, 'image/jpeg', 0.9)
  if (jpeg) return new File([jpeg], 'avatar.jpg', { type: 'image/jpeg' })
  throw new Error('生成头像失败，请重试')
}

interface CropperOptions {
  /** 画面边长（CSS 像素）。 */
  size: number
  /** 圆的直径（CSS 像素）。 */
  diameter: number
  onChange: () => void
}

/** 画布上的拖动、双指缩放、滚轮缩放、双击和方向键。 */
export class AvatarCropper {
  canvas: HTMLCanvasElement
  opts: CropperOptions
  src: Source | null = null
  state: CropState | null = null
  locked = false
  private active = false
  private pointers = new Map<number, { x: number; y: number }>()
  private lastTap = 0
  private dpr = Math.min(window.devicePixelRatio || 1, 3)
  private off: (() => void)[] = []

  constructor(canvas: HTMLCanvasElement, opts: CropperOptions) {
    this.canvas = canvas
    this.opts = opts
    canvas.width = canvas.height = Math.round(opts.size * this.dpr)
    canvas.style.width = canvas.style.height = `${opts.size}px`
    canvas.style.touchAction = 'none'
    this.bind()
    this.draw()
  }

  get diameter() {
    return this.opts.diameter
  }

  setImage(src: Source | null) {
    this.src = src
    this.state = src ? initialCrop(src.width, src.height) : null
    this.changed()
  }

  /** 换另一块画布时沿用已有的图和位置。 */
  restore(src: Source | null, state: CropState | null) {
    this.src = src
    this.state = state
    this.changed()
  }

  zoomTo(z: number, px = 0, py = 0) {
    if (this.state) this.state = zoomCrop(this.state, this.diameter, z, px, py)
    this.changed()
  }

  pan(dx: number, dy: number) {
    if (this.state) this.state = panCrop(this.state, this.diameter, dx, dy)
    this.changed()
  }

  reset() {
    if (this.src) this.state = initialCrop(this.src.width, this.src.height)
    this.changed()
  }

  rotate() {
    if (!this.src || !this.state) return
    this.src = rotateSource(this.src)
    this.state = rotateCrop(this.state, this.diameter)
    this.changed()
  }

  lock(locked: boolean) {
    this.locked = locked
    this.canvas.style.cursor = locked ? 'progress' : ''
    this.draw()
  }

  destroy() {
    this.off.forEach(f => f())
    this.off = []
  }

  private changed() {
    this.draw()
    this.opts.onChange()
  }

  draw() {
    const ctx = this.canvas.getContext('2d')
    if (!ctx) return
    const { size: S, diameter: D } = this.opts
    const styles = getComputedStyle(this.canvas)
    const dark = document.documentElement.dataset.theme === 'dark'
    ctx.setTransform(this.dpr, 0, 0, this.dpr, 0, 0)
    ctx.clearRect(0, 0, S, S)
    ctx.fillStyle = styles.getPropertyValue('--surface-2').trim() || '#f2f2f2'
    ctx.fillRect(0, 0, S, S)
    const c = S / 2, r = D / 2
    if (this.src && this.state) {
      const s = scaleOf(this.state, D), w = this.state.iw * s, h = this.state.ih * s
      ctx.imageSmoothingQuality = 'high'
      ctx.drawImage(this.src, c + this.state.tx - w / 2, c + this.state.ty - h / 2, w, h)
    }
    ctx.beginPath()
    ctx.rect(0, 0, S, S)
    ctx.arc(c, c, r, 0, Math.PI * 2, true)
    ctx.fillStyle = dark ? 'rgba(28,28,30,.74)' : 'rgba(250,250,250,.74)'
    ctx.fill('evenodd')
    if (this.active && this.src) {
      ctx.save()
      ctx.beginPath()
      ctx.arc(c, c, r, 0, Math.PI * 2)
      ctx.clip()
      ctx.strokeStyle = 'rgba(255,255,255,.75)'
      ctx.lineWidth = 1
      ctx.beginPath()
      for (const f of [1 / 3, 2 / 3]) {
        ctx.moveTo(c - r + D * f, c - r); ctx.lineTo(c - r + D * f, c + r)
        ctx.moveTo(c - r, c - r + D * f); ctx.lineTo(c + r, c - r + D * f)
      }
      ctx.stroke()
      ctx.restore()
    }
    ctx.beginPath(); ctx.arc(c, c, r + 1.5, 0, Math.PI * 2); ctx.strokeStyle = 'rgba(0,0,0,.10)'; ctx.lineWidth = 1; ctx.stroke()
    ctx.beginPath(); ctx.arc(c, c, r, 0, Math.PI * 2); ctx.strokeStyle = 'rgba(255,255,255,.95)'; ctx.lineWidth = 2; ctx.stroke()
  }

  private ratio() {
    return this.opts.size / this.canvas.getBoundingClientRect().width
  }

  private local(x: number, y: number) {
    const rc = this.canvas.getBoundingClientRect(), k = this.opts.size / rc.width
    return { x: (x - rc.left) * k - this.opts.size / 2, y: (y - rc.top) * k - this.opts.size / 2 }
  }

  private listen<K extends keyof HTMLElementEventMap>(type: K, fn: (e: HTMLElementEventMap[K]) => void, opts?: AddEventListenerOptions) {
    this.canvas.addEventListener(type, fn as EventListener, opts)
    this.off.push(() => this.canvas.removeEventListener(type, fn as EventListener, opts))
  }

  private bind() {
    this.listen('pointerdown', e => {
      if (this.locked || !this.state) return
      try { this.canvas.setPointerCapture(e.pointerId) } catch { /* 合成事件没有捕获 */ }
      this.pointers.set(e.pointerId, { x: e.clientX, y: e.clientY })
      this.active = true
      this.draw()
      const now = Date.now()
      if (this.pointers.size === 1 && now - this.lastTap < 300) {
        const p = this.local(e.clientX, e.clientY)
        this.zoomTo(doubleTapZoom(this.state.zoom), p.x, p.y)
        this.lastTap = 0
      } else this.lastTap = now
    })
    this.listen('pointermove', e => {
      if (!this.pointers.has(e.pointerId) || this.locked || !this.state) return
      const k = this.ratio()
      if (this.pointers.size === 1) {
        const o = this.pointers.get(e.pointerId)!
        this.pointers.set(e.pointerId, { x: e.clientX, y: e.clientY })
        this.pan((e.clientX - o.x) * k, (e.clientY - o.y) * k)
        return
      }
      const before = [...this.pointers.values()]
      this.pointers.set(e.pointerId, { x: e.clientX, y: e.clientY })
      const after = [...this.pointers.values()]
      const dist = (a: { x: number; y: number }, b: { x: number; y: number }) => Math.hypot(a.x - b.x, a.y - b.y)
      const mid = (a: { x: number; y: number }, b: { x: number; y: number }) => ({ x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 })
      const d0 = dist(before[0], before[1]), d1 = dist(after[0], after[1])
      const m0 = mid(before[0], before[1]), m1 = mid(after[0], after[1])
      let next = panCrop(this.state, this.diameter, (m1.x - m0.x) * k, (m1.y - m0.y) * k)
      const p = this.local(m1.x, m1.y)
      if (d0 > 0) next = zoomCrop(next, this.diameter, next.zoom * (d1 / d0), p.x, p.y)
      this.state = next
      this.changed()
    })
    const up = (e: PointerEvent) => {
      this.pointers.delete(e.pointerId)
      if (!this.pointers.size) { this.active = false; this.draw() }
    }
    this.listen('pointerup', up)
    this.listen('pointercancel', up)
    this.listen('wheel', e => {
      if (this.locked || !this.state) return
      e.preventDefault()
      const p = this.local(e.clientX, e.clientY)
      this.zoomTo(this.state.zoom * Math.exp(-e.deltaY * 0.0015), p.x, p.y)
    }, { passive: false })
    this.listen('keydown', e => {
      if (this.locked || !this.state) return
      const step = e.shiftKey ? 16 : 4
      const keys: Record<string, () => void> = {
        ArrowLeft: () => this.pan(-step, 0), ArrowRight: () => this.pan(step, 0),
        ArrowUp: () => this.pan(0, -step), ArrowDown: () => this.pan(0, step),
        '+': () => this.zoomTo(this.state!.zoom * 1.1), '=': () => this.zoomTo(this.state!.zoom * 1.1),
        '-': () => this.zoomTo(this.state!.zoom / 1.1), '0': () => this.reset(),
      }
      const run = keys[e.key]
      if (run) { e.preventDefault(); run() }
    })
  }
}
