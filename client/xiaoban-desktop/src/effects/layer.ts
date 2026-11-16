import "./layer.css";
import type { ArowanaRenderer } from "./arowana/renderer.js";
import { defaultAquarium, type AquariumMode } from './arowana/settings.ts';
import { drawEffects } from "./draw.ts";
import { ParticleSystem } from "./sim.ts";
import { defaultParams, isArowana, isHtmlEffect, type EffectPack, type EffectParams } from "./types.ts";

export interface LayerConfig { paused: boolean; fps: 30 | 60 }

/**
 * A transparent layer over `host` that shows one effect pack. Giving it a different pack starts that effect;
 * giving it the same pack with new parameters adjusts it in place, so the particles do not jump.
 */
export class EffectsLayer {
  private canvas: HTMLCanvasElement | null = null;
  private htmlFrame: HTMLIFrameElement | null = null;
  private context: CanvasRenderingContext2D | null = null;
  private observer: ResizeObserver | null = null;
  private system: ParticleSystem | null = null;
  private arowana: ArowanaRenderer | null = null;
  private requestedMode: AquariumMode | null = null;
  private loading: AbortController | null = null;
  private time = 0;
  private frame = 0;
  private last: number | null = null;
  private width = 0;
  private height = 0;
  private destroyed = false;
  private pending: Promise<void> = Promise.resolve();
  constructor(private host: HTMLElement, private pack: EffectPack | null, private params: EffectParams = { ...defaultParams },
    private config: LayerConfig = { paused: false, fps: 60 }, private random?: () => number,
    private onError?: (error: string) => void) {
    this.reconcile();
  }
  update(pack: EffectPack | null, params: EffectParams, config: LayerConfig = this.config) {
    if (this.destroyed) return;
    if (isArowana(this.pack) !== isArowana(pack) || isHtmlEffect(this.pack) !== isHtmlEffect(pack)) this.release();
    this.pack = pack; this.params = params; this.config = config;
    this.reconcile();
  }
  snapshot() {
    return {
      effect: this.pack?.id ?? "",
      canvasPresent: this.canvas !== null && !this.canvas.hidden,
      canvasWidth: this.canvas?.width ?? 0,
      canvasHeight: this.canvas?.height ?? 0,
      running: this.frame !== 0 || this.htmlFrame !== null,
      paused: this.config.paused,
      fps: this.config.fps,
      params: this.params,
      particles: this.arowana?.fishCount ? [this.arowana.fishCount] : this.system?.counts() ?? [],
    };
  }
  async whenReady(): Promise<void> {
    let pending;
    do { pending = this.pending; await pending; } while (pending !== this.pending);
  }
  destroy() { this.destroyed = true; this.release(); }

  private reconcile() {
    if (!this.pack) { this.release(); return; }
    if (isHtmlEffect(this.pack)) {
      this.reconcileHtml(this.pack);
      return;
    }
    const fish = isArowana(this.pack);
    if (!this.canvas) {
      const canvas = document.createElement("canvas");
      canvas.className = "effects-canvas";
      canvas.setAttribute("aria-hidden", "true");
      if (!fish) {
        this.context = canvas.getContext("2d", { alpha: true });
        if (!this.context) return;
      }
      this.canvas = canvas;
      this.host.classList.add("effects-host");
      this.host.append(canvas);
      if (fish) {
        canvas.hidden = true;
        this.host.classList.add("effects-loading");
        this.loading = new AbortController();
        canvas.addEventListener("webglcontextlost", this.onContextLost, { signal: this.loading.signal });
        this.pending = this.loadArowana(canvas, this.loading.signal);
      }
      if (typeof ResizeObserver !== "undefined") {
        this.observer = new ResizeObserver(this.resize);
        this.observer.observe(this.host);
      }
      window.addEventListener("resize", this.resize);
      window.addEventListener("pagehide", this.onPageHide);
      document.addEventListener("visibilitychange", this.onVisibility);
    }
    if (!fish && this.system?.pack !== this.pack) this.system = new ParticleSystem(this.pack, this.random);
    if (this.arowana) this.syncAquarium();
    this.resize();
    this.stop();
    if ((this.arowana?.mode || this.system) && !this.config.paused && !document.hidden) this.frame = requestAnimationFrame(this.tick);
  }
  private reconcileHtml(pack: EffectPack) {
    this.host.classList.add("effects-host");
    window.addEventListener("pagehide", this.onPageHide);
    document.addEventListener("visibilitychange", this.onVisibility);
    if (this.config.paused || document.hidden) {
      this.htmlFrame?.remove();
      this.htmlFrame = null;
      return;
    }
    const origin = /Windows/i.test(navigator.userAgent) ? "http://effect-html.localhost" : "effect-html://localhost";
    const src = `${origin}/${encodeURIComponent(pack.id)}?v=${pack.htmlRevision}`;
    if (this.htmlFrame?.getAttribute("src") === src) return;
    this.htmlFrame?.remove();
    const frame = document.createElement("iframe");
    frame.className = "effects-frame";
    frame.title = pack.name;
    frame.tabIndex = -1;
    frame.setAttribute("aria-hidden", "true");
    frame.setAttribute("sandbox", "allow-scripts");
    frame.referrerPolicy = "no-referrer";
    frame.src = src;
    this.host.append(frame);
    this.htmlFrame = frame;
  }
  private async loadArowana(canvas: HTMLCanvasElement, signal: AbortSignal) {
    try {
      const { createArowanaRenderer } = await import("./arowana/renderer.js");
      if (signal.aborted) return;
      this.arowana = createArowanaRenderer(canvas);
      this.reconcile();
    } catch (error) {
      if (!signal.aborted) this.failArowana(error);
    }
  }
  private syncAquarium() {
    const renderer = this.arowana!;
    const settings = this.params.aquarium ?? defaultAquarium;
    renderer.setLighting(settings.lighting);
    renderer.setAquascape(settings.aquascape);
    this.host.classList.toggle("effects-aquascape", settings.aquascape);
    if (settings.mode === this.requestedMode) return;
    this.requestedMode = settings.mode;
    // Keep drawing the previous set until the newly requested one is ready.
    const pending = renderer.setMode(settings.mode).then(() => {
      if (this.pending !== pending || this.arowana !== renderer || !this.canvas) return;
      this.canvas.hidden = false;
      this.host.classList.remove("effects-loading");
      this.reconcile();
    }).catch(error => {
      if (this.pending === pending && this.arowana === renderer) this.failArowana(error);
    });
    this.pending = pending;
  }
  private failArowana(error: unknown) {
    this.release();
    this.onError?.(`龙鱼巡游无法显示：${error instanceof Error ? error.message : String(error)}`);
  }
  private onContextLost = () => this.failArowana("图形连接已中断，请重新选择特效。");
  private resize = () => {
    if (!this.canvas) return;
    const rect = this.host.getBoundingClientRect();
    this.width = Math.max(1, rect.width); this.height = Math.max(1, rect.height);
    const dpr = Math.max(1, Math.min(window.devicePixelRatio || 1, 2));
    if (this.arowana) this.arowana.resize(this.width, this.height, dpr, this.params.size);
    else if (this.context && this.system) {
      this.canvas.width = Math.round(this.width * dpr); this.canvas.height = Math.round(this.height * dpr);
      this.context.setTransform(dpr, 0, 0, dpr, 0, 0);
      this.system.configure(this.width, this.height, this.params);
    }
    this.draw();
  };
  private tick = (now: number) => {
    this.frame = 0;
    if ((!this.system && !this.arowana?.mode) || this.config.paused || document.hidden || this.destroyed) return;
    const elapsed = this.last === null ? 1 / this.config.fps : (now - this.last) / 1000;
    if (this.last === null || elapsed >= 1 / this.config.fps - 0.001) {
      this.system?.advance(elapsed);
      if (this.arowana) this.time += Math.min(elapsed, 0.1) * this.params.speed;
      this.draw();
      this.last = now;
    }
    if (this.canvas) this.frame = requestAnimationFrame(this.tick);
  };
  private draw() {
    if (this.arowana?.mode) this.arowana.render(this.time);
    else if (this.context && this.system) drawEffects(this.context, this.system);
  }
  private stop() {
    if (this.frame) cancelAnimationFrame(this.frame);
    this.frame = 0; this.last = null;
  }
  private onVisibility = () => this.reconcile();
  private onPageHide = () => this.destroy();
  private release() {
    this.stop();
    this.loading?.abort(); this.loading = null;
    this.pending = Promise.resolve();
    this.arowana?.destroy(); this.arowana = null; this.requestedMode = null; this.time = 0;
    this.observer?.disconnect(); this.observer = null;
    window.removeEventListener("resize", this.resize);
    window.removeEventListener("pagehide", this.onPageHide);
    document.removeEventListener("visibilitychange", this.onVisibility);
    this.htmlFrame?.remove(); this.htmlFrame = null;
    this.canvas?.remove(); this.canvas = null; this.context = null; this.system = null;
    this.host.classList.remove("effects-host", "effects-loading", "effects-aquascape");
  }
}
