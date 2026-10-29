import { desktop, invoke } from "./bridge.ts";
import { EffectsLayer } from "./effects/layer.ts";
import { normalizeParams, parsePack } from "./effects/parse.ts";
import type { EffectPack } from "./effects/types.ts";

interface Payload {
  /** The pack's data, exactly as read from its file; it is checked here before it is drawn. */
  effect: unknown;
  params: unknown;
  paused: boolean;
  fps: 30 | 60;
  test?: boolean;
}
declare global {
  interface Window {
    __XIAOBAN_EFFECTS__?: Payload;
  }
}

/**
 * The desktop-wide particle layer. It is a transparent page in a click-through window that sits over the
 * desktop picture; the host sends it the chosen effect and the global pause and frame rate.
 */
export async function mountEffects() {
  let state: Payload = window.__XIAOBAN_EFFECTS__ ?? { effect: null, params: {}, paused: false, fps: 60 };
  document.body.className = "effects-scene";
  document.body.innerHTML = '<main class="effects-stage"></main>';
  const host = document.querySelector<HTMLElement>(".effects-stage")!;
  // The host repeats the pack with every settings change. The same data keeps the same pack, so a changed
  // slider adjusts the running effect instead of starting it over.
  let lastKey = "", pack: EffectPack | null = null;
  const resolve = (): EffectPack | null => {
    const key = JSON.stringify(state.effect ?? null);
    if (key !== lastKey) {
      lastKey = key;
      const parsed = parsePack(state.effect);
      pack = parsed.ok ? parsed.pack : null;
    }
    return pack;
  };
  const layer = new EffectsLayer(host, resolve(), normalizeParams(state.params), { paused: state.paused, fps: state.fps });
  const lifecycle = new AbortController();
  window.addEventListener("effects-config", (event) => {
    state = { ...state, ...(event as CustomEvent<Partial<Payload>>).detail };
    layer.update(resolve(), normalizeParams(state.params), { paused: state.paused, fps: state.fps });
  }, { signal: lifecycle.signal });
  const dispose = () => { lifecycle.abort(); layer.destroy(); };
  window.addEventListener("pagehide", dispose, { once: true });
  window.addEventListener("beforeunload", dispose, { once: true });
  const report: Record<string, unknown> = {};
  if (state.test && desktop) {
    // In the isolated test profile, prove this page cannot reach commands that change the desktop.
    let blocked = false;
    try { await invoke("set_autostart", { enabled: false }); } catch { blocked = true; }
    report.privilegedIpcBlocked = blocked;
  }
  if (desktop) await invoke("scene_ready", { report: { kind: "effects", effects: layer.snapshot(), ...report } });
  else document.body.dataset.ready = "true";
}
