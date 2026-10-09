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
      supported: this.kind === "image",
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
    if (this.kind !== "image" || !effectNames.some((name) =>
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
    ctx.lineWidth = 1;
    for (const name of effectNames) {
      const intensity = this.options[name].intensity;
      if (name === "rain") {
        ctx.strokeStyle = `rgba(195,225,245,${0.25 + intensity * 0.35})`;
        ctx.beginPath();
        for (const p of this.particles.particles.rain) {
          const x = p.x * this.width, y = p.y * this.height;
          ctx.moveTo(x, y); ctx.lineTo(x + 3 * p.size, y + 22 * p.size);
        }
        ctx.stroke();
      } else for (const p of this.particles.particles[name]) {
        const x = p.x * this.width, y = p.y * this.height;
        if (name === "snow") {
          ctx.fillStyle = `rgba(255,255,255,${0.45 + intensity * 0.45})`;
          ctx.beginPath(); ctx.arc(x, y, 2.2 * p.size, 0, Math.PI * 2); ctx.fill();
        } else {
          const glow = (0.5 + Math.sin(p.phase) * 0.3) * (0.4 + intensity * 0.6);
          ctx.fillStyle = `rgba(218,255,121,${glow * 0.16})`;
          ctx.beginPath(); ctx.arc(x, y, 8 * p.size, 0, Math.PI * 2); ctx.fill();
          ctx.fillStyle = `rgba(242,255,173,${glow})`;
          ctx.beginPath(); ctx.arc(x, y, 1.8 * p.size, 0, Math.PI * 2); ctx.fill();
        }
      }
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
