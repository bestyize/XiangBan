import "./wallpaper-effects.css";

export const effectNames = ["rain", "snow", "fireflies"] as const;
export type EffectName = (typeof effectNames)[number];
export interface EffectOptions {
  enabled: boolean;
  intensity: number;
  speed: number;
}
export interface WallpaperEffectsOptions {
  rain: EffectOptions;
  snow: EffectOptions;
  fireflies: EffectOptions;
}
export interface EffectsAnimationConfig { paused: boolean; fps: 30 | 60 }
export const wallpaperEffectsDefaults: WallpaperEffectsOptions = {
  rain: { enabled: false, intensity: 0.5, speed: 1 },
  snow: { enabled: false, intensity: 0.5, speed: 1 },
  fireflies: { enabled: false, intensity: 0.5, speed: 1 },
};
const number = (value: unknown, fallback: number, min: number, max: number) =>
  typeof value === "number" && Number.isFinite(value)
    ? Math.max(min, Math.min(max, value)) : fallback;
export function normalizeWallpaperEffects(value?: unknown): WallpaperEffectsOptions {
  const source = value && typeof value === "object" ? value as Record<string, unknown> : {};
  let selected = false;
  return Object.fromEntries(effectNames.map((name) => {
    const raw = source[name];
    const effect = raw && typeof raw === "object" ? raw as Record<string, unknown> : {};
    const enabled = effect.enabled === true && !selected;
    if (enabled) selected = true;
    return [name, {
      enabled,
      intensity: number(effect.intensity, 0.5, 0, 1),
      speed: number(effect.speed, 1, 0.25, 3),
    }];
  })) as unknown as WallpaperEffectsOptions;
}

export interface EffectParticle { x: number; y: number; size: number; phase: number; drift: number }
/** Pure simulation, shared by the small preview and the native media scene. */
export class EffectParticles {
  readonly particles: Record<EffectName, EffectParticle[]> = { rain: [], snow: [], fireflies: [] };
  constructor(private random: () => number = Math.random) {}
  configure(options: WallpaperEffectsOptions, width: number, height: number) {
    const area = Math.max(0.15, Math.min(1.5, width * height / (1920 * 1080)));
    for (const name of effectNames) {
      const option = options[name];
      const limit = { rain: 400, snow: 220, fireflies: 65 }[name];
      const count = option.enabled && option.intensity > 0
        ? Math.ceil(limit * area * option.intensity) : 0;
      const list = this.particles[name];
      list.length = Math.min(list.length, count);
      while (list.length < count) list.push({
        x: this.random(), y: this.random(), size: 0.5 + this.random(),
        phase: this.random() * Math.PI * 2, drift: this.random() * 2 - 1,
      });
    }
  }
  advance(seconds: number, options: WallpaperEffectsOptions, width: number, height: number) {
    if (width <= 0 || height <= 0) return;
    for (const name of effectNames) for (const p of this.particles[name]) {
      const time = Math.max(0, Math.min(seconds, 0.1)) * options[name].speed;
      p.phase += time * (name === "fireflies" ? 1.2 : 0.7);
      if (name === "rain") {
        p.x += time * 140 * p.size / width;
        p.y += time * 850 * p.size / height;
      } else if (name === "snow") {
        p.x += time * (Math.sin(p.phase) * 24 + p.drift * 12) / width;
        p.y += time * 48 * p.size / height;
      } else {
        p.x += time * (Math.cos(p.phase) * 16 + p.drift * 6) / width;
        p.y += time * Math.sin(p.phase * 0.73) * 13 / height;
      }
      p.x = (p.x + 1) % 1;
      p.y = (p.y + 1) % 1;
    }
  }
}

/** Trusted host overlay; it grants no additional capability to a wallpaper iframe. */
export class WallpaperEffects {
  private options: WallpaperEffectsOptions;
  private config: EffectsAnimationConfig;
  private kind: string;
  private canvas: HTMLCanvasElement | null = null;
  private context: CanvasRenderingContext2D | null = null;
  private observer: ResizeObserver | null = null;
  private particles = new EffectParticles();
  private frame = 0;
  private last: number | null = null;
  private width = 0;
  private height = 0;
  private destroyed = false;
  constructor(private host: HTMLElement, kind: string, options?: unknown,
    config: EffectsAnimationConfig = { paused: false, fps: 60 }) {
    this.kind = kind;
    this.options = normalizeWallpaperEffects(options);
    this.config = config;
    this.reconcile();
  }
  update(kind: string, options?: unknown,
    config: EffectsAnimationConfig = this.config) {
    if (this.destroyed) return;
    this.kind = kind;
    this.options = normalizeWallpaperEffects(options);
    this.config = config;
    this.reconcile();
  }
  snapshot() {
    return {
      supported: ["image", "video"].includes(this.kind),
      canvasPresent: this.canvas !== null,
      canvasWidth: this.canvas?.width ?? 0,
      canvasHeight: this.canvas?.height ?? 0,
      running: this.frame !== 0,
      paused: this.config.paused,
      fps: this.config.fps,
      options: normalizeWallpaperEffects(this.options),
      particles: Object.fromEntries(effectNames.map((name) => [name, this.particles.particles[name].length])),
    };
  }
  private reconcile() {
    if (!["image", "video"].includes(this.kind) || !effectNames.some((name) =>
      this.options[name].enabled && this.options[name].intensity > 0)) {
      this.release();
      return;
    }
    if (!this.canvas) {
      this.canvas = document.createElement("canvas");
      this.canvas.className = "wallpaper-effects";
      this.canvas.setAttribute("aria-hidden", "true");
      this.context = this.canvas.getContext("2d", { alpha: true });
      if (!this.context) { this.release(); return; }
      this.host.classList.add("wallpaper-effects-host");
      this.host.append(this.canvas);
      if (typeof ResizeObserver !== "undefined") {
        this.observer = new ResizeObserver(this.resize);
        this.observer.observe(this.host);
      }
      window.addEventListener("resize", this.resize);
      window.addEventListener("pagehide", this.onPageHide);
      document.addEventListener("visibilitychange", this.onVisibility);
    }
    this.resize();
    this.stop();
    this.draw();
    if (!this.config.paused && !document.hidden) this.frame = requestAnimationFrame(this.tick);
  }
  private resize = () => {
    if (!this.canvas || !this.context) return;
    const rect = this.host.getBoundingClientRect();
    this.width = Math.max(1, rect.width);
    this.height = Math.max(1, rect.height);
    const dpr = Math.max(1, Math.min(window.devicePixelRatio || 1, 2));
    this.canvas.width = Math.round(this.width * dpr);
    this.canvas.height = Math.round(this.height * dpr);
    this.context.setTransform(dpr, 0, 0, dpr, 0, 0);
    this.particles.configure(this.options, this.width, this.height);
    this.draw();
  };
  private tick = (now: number) => {
    this.frame = 0;
    if (!this.canvas || this.config.paused || document.hidden || this.destroyed) return;
    const elapsed = this.last === null ? 1 / this.config.fps : (now - this.last) / 1000;
    if (this.last === null || elapsed >= 1 / this.config.fps - 0.001) {
      this.particles.advance(elapsed, this.options, this.width, this.height);
      this.draw();
      this.last = now;
    }
    this.frame = requestAnimationFrame(this.tick);
  };
  private draw() {
    const ctx = this.context;
    if (!ctx) return;
    ctx.clearRect(0, 0, this.width, this.height);
    for (const name of effectNames) for (const p of this.particles.particles[name]) {
      const x = p.x * this.width, y = p.y * this.height;
      const depth = (p.size - 0.5) / 1;
      ctx.save();
      if (name === "rain") {
        // Fine distant rain and brighter foreground drops with fading tails.
        const length = 10 + depth * 24, slant = length * 0.165;
        const tail = ctx.createLinearGradient(x - slant, y - length, x, y);
        tail.addColorStop(0, 'rgba(205,230,250,0)');
        tail.addColorStop(0.72, `rgba(210,233,250,${0.2 + depth * 0.32})`);
        tail.addColorStop(1, `rgba(240,249,255,${0.35 + depth * 0.45})`);
        ctx.strokeStyle = tail; ctx.lineWidth = 0.6 + depth * 1.05; ctx.lineCap = 'round';
        ctx.beginPath(); ctx.moveTo(x - slant, y - length); ctx.lineTo(x, y); ctx.stroke();
      } else if (name === "snow") {
        const radius = 1.2 + depth * 3.8;
        ctx.translate(x, y); ctx.rotate(p.phase * (p.drift < 0 ? -1 : 1));
        ctx.globalAlpha = 0.4 + depth * 0.55;
        ctx.fillStyle = '#f7fbff'; ctx.strokeStyle = '#f7fbff';
        if (depth < 0.35) {
          ctx.beginPath(); ctx.arc(0, 0, radius, 0, Math.PI * 2); ctx.fill();
        } else {
          ctx.lineWidth = 0.6 + depth * 0.5; ctx.lineCap = 'round'; ctx.beginPath();
          for (let arm = 0; arm < 6; arm++) {
            const angle = arm * Math.PI / 3;
            const dx = Math.cos(angle), dy = Math.sin(angle);
            ctx.moveTo(0, 0); ctx.lineTo(dx * radius, dy * radius);
            for (const side of [-1, 1]) {
              const branch = angle + side * Math.PI / 3;
              ctx.moveTo(dx * radius * 0.6, dy * radius * 0.6);
              ctx.lineTo(dx * radius * 0.6 + Math.cos(branch) * radius * 0.32,
                         dy * radius * 0.6 + Math.sin(branch) * radius * 0.32);
            }
          }
          ctx.stroke();
        }
      } else {
        const pulse = Math.pow(0.5 + 0.5 * Math.sin(p.phase), 2);
        const glow = 0.16 + pulse * 0.8;
        const radius = 9 + depth * 9;
        const halo = ctx.createRadialGradient(x, y, 0, x, y, radius);
        halo.addColorStop(0, `rgba(234,255,155,${glow * 0.65})`);
        halo.addColorStop(0.2, `rgba(193,244,74,${glow * 0.35})`);
        halo.addColorStop(1, 'rgba(174,232,65,0)');
        ctx.fillStyle = halo; ctx.beginPath(); ctx.arc(x, y, radius, 0, Math.PI * 2); ctx.fill();
        ctx.translate(x, y); ctx.rotate(p.phase * 0.3);
        ctx.fillStyle = `rgba(54,72,34,${0.45 + pulse * 0.2})`;
        ctx.beginPath(); ctx.ellipse(0, -1.8 * p.size, 0.8 * p.size, 1.6 * p.size, 0, 0, Math.PI * 2); ctx.fill();
        ctx.fillStyle = `rgba(248,255,182,${glow})`;
        ctx.beginPath(); ctx.ellipse(0, 0, 1.2 * p.size, 1.6 * p.size, 0, 0, Math.PI * 2); ctx.fill();
      }
      ctx.restore();
    }
  }

  private stop() {
    if (this.frame) cancelAnimationFrame(this.frame);
    this.frame = 0;
    this.last = null;
  }
  private onVisibility = () => this.reconcile();
  private onPageHide = () => this.destroy();
  private release() {
    this.stop();
    this.observer?.disconnect(); this.observer = null;
    window.removeEventListener("resize", this.resize);
    window.removeEventListener("pagehide", this.onPageHide);
    document.removeEventListener("visibilitychange", this.onVisibility);
    this.canvas?.remove(); this.canvas = null; this.context = null;
    this.host.classList.remove("wallpaper-effects-host");
    this.particles = new EffectParticles();
  }
  destroy() { this.destroyed = true; this.release(); }
}
