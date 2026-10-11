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
    ctx.globalCompositeOperation = state.spec.blend === "add" ? "lighter" : "source-over";
    for (let i = 0; i < state.active; i++) {
      const p = state.order[i];
      if (p.flash > 0) drawFlash(ctx, state.spec, p);
      if (!p.visible || p.alpha <= 0.003) continue;
      ctx.save();
      drawParticle(ctx, system, state.spec, p);
      ctx.restore();
    }
  }
  ctx.globalCompositeOperation = "source-over";
}

/** The light where a firework bursts: white at the heart, the shell's colour around it. */
function drawFlash(ctx: Ctx, spec: Layer, p: Particle) {
  const { base } = spec.colors[p.color];
  const light = ctx.createRadialGradient(p.fx, p.fy, 0, p.fx, p.fy, p.fr);
  light.addColorStop(0, rgba([255, 255, 255], 0.9 * p.flash));
  light.addColorStop(0.18, rgba(base, 0.5 * p.flash));
  light.addColorStop(1, rgba(base, 0));
  ctx.fillStyle = light;
  ctx.beginPath(); ctx.arc(p.fx, p.fy, p.fr, 0, TAU); ctx.fill();
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
      if (shape.aspect !== 1) {
        // Drawn round in a squeezed frame, so the light is an ellipse along the particle's angle.
        ctx.translate(p.x, p.y); ctx.rotate(p.angle); ctx.scale(1, shape.aspect); ctx.translate(-p.x, -p.y);
      }
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
      const length = lerp(shape.length[0], shape.length[1], p.d) * system.unit * system.params.size * p.stretch;
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
    case "ring": {
      const scale = system.unit * system.params.size;
      const width = lerp(shape.width[0], shape.width[1], p.d) * scale, radius = p.size / 2;
      ctx.strokeStyle = rgba(base, 1); ctx.translate(p.x, p.y); ctx.rotate(p.angle); ctx.scale(1, shape.aspect);
      // A wide faint ring under the sharp one, so the edge of the water looks soft.
      ctx.globalAlpha = p.alpha * 0.3; ctx.lineWidth = width * 3.5;
      ctx.beginPath(); ctx.arc(0, 0, radius, 0, TAU); ctx.stroke();
      ctx.globalAlpha = p.alpha; ctx.lineWidth = width; ctx.strokeStyle = rgba(tip, 1);
      ctx.beginPath(); ctx.arc(0, 0, radius, 0, TAU); ctx.stroke();
      return;
    }
    case "beam": {
      const length = lerp(shape.length[0], shape.length[1], p.d) * system.unit * system.params.size;
      ctx.translate(p.x, p.y); ctx.rotate(p.angle); ctx.scale(p.size / 2 * p.scaleX, length / 2);
      // The bright spot sits low inside an ellipse, so the light rises from a sharp foot and fades upwards.
      const ray = ctx.createRadialGradient(0, shape.focus, 0, 0, 0, 1);
      ray.addColorStop(0, rgba(tip, p.alpha));
      ray.addColorStop(0.45, rgba(base, p.alpha * 0.55));
      ray.addColorStop(1, rgba(base, 0));
      ctx.fillStyle = ray; ctx.fillRect(-1, -1, 2, 2);
      return;
    }
    case "path": {
      ctx.translate(p.x, p.y); ctx.rotate(p.angle); ctx.scale(p.size / 2 * p.scaleX, p.size / 2);
      if (shape.halo) {
        const { scale, y, color, alpha } = shape.halo, light = ctx.createRadialGradient(0, y, 0, 0, y, scale);
        light.addColorStop(0, rgba(color, p.alpha * alpha)); light.addColorStop(0.4, rgba(color, p.alpha * alpha * 0.35)); light.addColorStop(1, rgba(color, 0));
        ctx.fillStyle = light; ctx.fillRect(-scale, y - scale, scale * 2, scale * 2);
      }
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
