import type { Particle, ParticleSystem } from "./sim.ts";
import type { Layer, Rgb } from "./types.ts";

type Ctx = CanvasRenderingContext2D;
const TAU = Math.PI * 2;
const lerp = (a: number, b: number, t: number) => a + (b - a) * t;
const rgba = (color: Rgb, alpha: number) => `rgba(${color[0]},${color[1]},${color[2]},${alpha})`;

const paths = new Map<string, Path2D>();
/** SVG path data is parsed once; a pack has a handful of shapes, so the cache stays small. */
function pathOf(d: string) {
  let path = paths.get(d);
  if (!path) {
    if (paths.size > 48) paths.clear();
    path = new Path2D(d);
    paths.set(d, path);
  }
  return path;
}

/** Draws every visible particle, far layers and far particles first. */
export function drawEffects(ctx: Ctx, system: ParticleSystem) {
  ctx.clearRect(0, 0, system.width, system.height);
  for (const state of system.layers) {
    for (let i = 0; i < state.active; i++) {
      const p = state.order[i];
      if (!p.visible || p.alpha <= 0.003) continue;
      ctx.save();
      drawParticle(ctx, system, state.spec, p);
      ctx.restore();
    }
  }
}

function drawParticle(ctx: Ctx, system: ParticleSystem, spec: Layer, p: Particle) {
  const { base, tip } = spec.colors[p.color];
  const shape = spec.shape;
  switch (shape.kind) {
    case "circle":
      ctx.fillStyle = rgba(base, p.alpha);
      ctx.beginPath(); ctx.arc(p.x, p.y, p.size / 2, 0, TAU); ctx.fill();
      return;
    case "glow": {
      const radius = p.size / 2;
      const halo = ctx.createRadialGradient(p.x, p.y, 0, p.x, p.y, radius);
      halo.addColorStop(0, rgba(base, p.alpha * 0.9));
      halo.addColorStop(0.25, rgba(tip, p.alpha * 0.5));
      halo.addColorStop(1, rgba(tip, 0));
      ctx.fillStyle = halo;
      ctx.beginPath(); ctx.arc(p.x, p.y, radius, 0, TAU); ctx.fill();
      if (shape.core > 0) {
        ctx.fillStyle = rgba(base, p.alpha);
        ctx.beginPath(); ctx.arc(p.x, p.y, Math.max(0.8, radius * shape.core), 0, TAU); ctx.fill();
      }
      return;
    }
    case "streak": {
      const scale = system.unit * (0.7 + 0.3 * system.params.size);
      const length = lerp(shape.length[0], shape.length[1], p.d) * system.unit * system.params.size;
      const width = lerp(shape.width[0], shape.width[1], p.d) * scale;
      const tailX = p.x - p.dx * length, tailY = p.y - p.dy * length;
      const tail = ctx.createLinearGradient(p.x, p.y, tailX, tailY);
      tail.addColorStop(0, rgba(base, p.alpha));
      tail.addColorStop(0.35, rgba(tip, p.alpha * 0.5));
      tail.addColorStop(1, rgba(tip, 0));
      ctx.lineCap = "round"; ctx.strokeStyle = tail;
      if (shape.head > 0) {
        // A wider, fainter line under the bright one gives the tail its glow.
        ctx.globalAlpha = 0.28; ctx.lineWidth = width * 3.2;
        ctx.beginPath(); ctx.moveTo(p.x, p.y); ctx.lineTo(tailX, tailY); ctx.stroke();
        ctx.globalAlpha = 1;
      }
      ctx.lineWidth = width;
      ctx.beginPath(); ctx.moveTo(p.x, p.y); ctx.lineTo(tailX, tailY); ctx.stroke();
      if (shape.head > 0) {
        const radius = width * shape.head * 2;
        const glow = ctx.createRadialGradient(p.x, p.y, 0, p.x, p.y, radius);
        glow.addColorStop(0, rgba(base, p.alpha * 0.95));
        glow.addColorStop(0.35, rgba(tip, p.alpha * 0.4));
        glow.addColorStop(1, rgba(tip, 0));
        ctx.fillStyle = glow;
        ctx.beginPath(); ctx.arc(p.x, p.y, radius, 0, TAU); ctx.fill();
      }
      return;
    }
    case "path": {
      ctx.translate(p.x, p.y); ctx.rotate(p.angle); ctx.scale(p.size / 2 * p.scaleX, p.size / 2);
      ctx.globalAlpha = p.alpha; ctx.lineCap = "round"; ctx.lineJoin = "round";
      const path = pathOf(shape.d);
      if (shape.style !== "stroke") {
        const fill = ctx.createLinearGradient(0, -1, 0, 1);
        fill.addColorStop(0, rgba(base, 1)); fill.addColorStop(1, rgba(tip, 1));
        ctx.fillStyle = fill; ctx.fill(path);
      }
      if (shape.style !== "fill") { ctx.strokeStyle = rgba(base, 1); ctx.lineWidth = shape.lineWidth; ctx.stroke(path); }
      if (shape.vein) { ctx.strokeStyle = rgba(shape.vein.color, shape.vein.alpha); ctx.lineWidth = shape.vein.width; ctx.stroke(pathOf(shape.vein.d)); }
    }
  }
}
