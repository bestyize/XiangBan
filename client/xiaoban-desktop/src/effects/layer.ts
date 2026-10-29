import "./layer.css";
import { drawEffects } from "./draw.ts";
import { ParticleSystem } from "./sim.ts";
import { defaultParams, type EffectPack, type EffectParams } from "./types.ts";

export interface LayerConfig { paused: boolean; fps: 30 | 60 }

/**
 * A transparent canvas over `host` that shows one effect pack. Giving it a different pack starts that effect;
 * giving it the same pack with new parameters adjusts it in place, so the particles do not jump.
 */
export class EffectsLayer {
  private canvas: HTMLCanvasElement | null = null;
  private context: CanvasRenderingContext2D | null = null;
  private observer: ResizeObserver | null = null;
  private system: ParticleSystem | null = null;
  private frame = 0;
  private last: number | null = null;
  private width = 0;
  private height = 0;
  private destroyed = false;
  constructor(private host: HTMLElement, private pack: EffectPack | null, private params: EffectParams = { ...defaultParams },
    private config: LayerConfig = { paused: false, fps: 60 }, private random?: () => number) {
    this.reconcile();
  }
  update(pack: EffectPack | null, params: EffectParams, config: LayerConfig = this.config) {
    if (this.destroyed) return;
    this.pack = pack; this.params = params; this.config = config;
    this.reconcile();
  }
  snapshot() {
    return {
      effect: this.pack?.id ?? "",
      canvasPresent: this.canvas !== null,
      canvasWidth: this.canvas?.width ?? 0,
      canvasHeight: this.canvas?.height ?? 0,
      running: this.frame !== 0,
      paused: this.config.paused,
      fps: this.config.fps,
      params: this.params,
      particles: this.system?.counts() ?? [],
    };
  }
  destroy() { this.destroyed = true; this.release(); }

  private reconcile() {
    if (!this.pack) { this.release(); return; }
    if (!this.canvas) {
      const canvas = document.createElement("canvas");
      canvas.className = "effects-canvas";
      canvas.setAttribute("aria-hidden", "true");
      const context = canvas.getContext("2d", { alpha: true });
      if (!context) return;
      this.canvas = canvas; this.context = context;
      this.host.classList.add("effects-host");
      this.host.append(canvas);
      if (typeof ResizeObserver !== "undefined") {
        this.observer = new ResizeObserver(this.resize);
        this.observer.observe(this.host);
      }
      window.addEventListener("resize", this.resize);
      window.addEventListener("pagehide", this.onPageHide);
      document.addEventListener("visibilitychange", this.onVisibility);
    }
    if (!this.system || this.system.pack !== this.pack) this.system = new ParticleSystem(this.pack, this.random);
    this.resize();
    this.stop();
    if (!this.config.paused && !document.hidden) this.frame = requestAnimationFrame(this.tick);
  }
  private resize = () => {
    if (!this.canvas || !this.context || !this.system) return;
    const rect = this.host.getBoundingClientRect();
    this.width = Math.max(1, rect.width); this.height = Math.max(1, rect.height);
    const dpr = Math.max(1, Math.min(window.devicePixelRatio || 1, 2));
    this.canvas.width = Math.round(this.width * dpr); this.canvas.height = Math.round(this.height * dpr);
    this.context.setTransform(dpr, 0, 0, dpr, 0, 0);
    this.system.configure(this.width, this.height, this.params);
    this.draw();
  };
  private tick = (now: number) => {
    this.frame = 0;
    if (!this.canvas || !this.system || this.config.paused || document.hidden || this.destroyed) return;
    const elapsed = this.last === null ? 1 / this.config.fps : (now - this.last) / 1000;
    if (this.last === null || elapsed >= 1 / this.config.fps - 0.001) {
      this.system.advance(elapsed);
      this.draw();
      this.last = now;
    }
    this.frame = requestAnimationFrame(this.tick);
  };
  private draw() { if (this.context && this.system) drawEffects(this.context, this.system); }
  private stop() {
    if (this.frame) cancelAnimationFrame(this.frame);
    this.frame = 0; this.last = null;
  }
  private onVisibility = () => this.reconcile();
  private onPageHide = () => this.destroy();
  private release() {
    this.stop();
    this.observer?.disconnect(); this.observer = null;
    window.removeEventListener("resize", this.resize);
    window.removeEventListener("pagehide", this.onPageHide);
    document.removeEventListener("visibilitychange", this.onVisibility);
    this.canvas?.remove(); this.canvas = null; this.context = null; this.system = null;
    this.host.classList.remove("effects-host");
  }
}
