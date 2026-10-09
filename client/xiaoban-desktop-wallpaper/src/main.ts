import {
  desktop,
  defaults,
  invoke,
  listen,
  type Settings,
  type CursorSample,
} from "./bridge.ts";
import { PetView } from "./pet.ts";
const room =
  '<div class="room"><div class="room-glow"></div><div class="room-window"><div class="moon"></div><div class="stars"></div><div class="skyline"></div></div><div class="plant"><i></i><i></i><i></i><i></i><b></b></div><div class="desk"></div><div class="room-floor"></div></div>';
async function init() {
  const mode = new URLSearchParams(location.search).get("scene");
  if (!mode) {
    const { mountControl } = await import("./control.ts");
    await mountControl();
    return;
  }
  await import("./style.css");
  if (mode === "games") {
    const { mountGameSpace } = await import("./games.ts");
    await mountGameSpace();
    return;
  }
  if (mode === "care") {
    const { mountCareBubble } = await import("./care-bubble.ts");
    await mountCareBubble();
    return;
  }
  if (mode === "content") {
    const { mountContent } = await import("./content.ts");
    await mountContent();
    return;
  }
  let config = { ...defaults };
  document.body.className = mode;
  document.querySelector<HTMLElement>("#app")!.innerHTML =
    mode === "wallpaper"
      ? `<section class="scene full-scene">${room}<div class="wallpaper-pet"></div><div class="scene-caption"><span>小伴</span><p>陪你，把日子过慢一点。</p></div></section>`
      : '<section class="floating-stage"></section>';
  const pet = new PetView(
    document.querySelector<HTMLElement>(
      mode === "wallpaper" ? ".wallpaper-pet" : ".floating-stage",
    )!,
    config,
  );
  let pointer: { x: number; y: number; dragged: boolean } | null = null,
    hovering: boolean | null = null;
  pet.host.addEventListener("pointerdown", (event) => {
    if (
      event.button !== 0 ||
      mode !== "floating" ||
      !pet.hit(event.clientX, event.clientY)
    )
      return;
    pointer = { x: event.clientX, y: event.clientY, dragged: false };
    pet.host.setPointerCapture(event.pointerId);
  });
  pet.host.addEventListener("pointermove", (event) => {
    if (
      pointer &&
      !pointer.dragged &&
      Math.hypot(event.clientX - pointer.x, event.clientY - pointer.y) > 5
    ) {
      pointer.dragged = true;
      invoke("start_drag").catch(console.error);
    }
  });
  pet.host.addEventListener("pointerup", (event) => {
    if (pointer && !pointer.dragged && pet.hit(event.clientX, event.clientY))
      pet.poke();
    pointer = null;
  });
  pet.host.addEventListener("pointercancel", () => (pointer = null));
  pet.host.addEventListener("contextmenu", (event) => {
    event.preventDefault();
    invoke("show_control").catch(console.error);
  });
  await listen<CursorSample>("desktop-cursor", (sample) => {
    if (!config.companionEnabled) return;
    pet.look(sample.x, sample.y, sample.moved);
    const hit = pet.hit(sample.x, sample.y);
    if (mode === "floating" && hit !== hovering) {
      hovering = hit;
      invoke("set_hit_test", { hit }).catch(console.error);
    }
    if (mode === "wallpaper" && sample.pressed && sample.desktop && hit)
      pet.poke();
  });
  const apply = (settings: Settings) => {
    config = settings;
    document.body.dataset.theme = config.theme;
    document.body.dataset.paused = String(config.paused);
    pet.host.hidden = !config.companionEnabled;
    pet.update({ ...config, paused: config.paused || !config.companionEnabled });
  };
  await listen<Settings>("settings-changed", apply);
  await listen("pet-poke", () => pet.poke());
  if (desktop) {
    apply(await invoke<Settings>("get_settings"));
    await invoke("scene_ready");
  } else apply(config);
  window.addEventListener("beforeunload", () => pet.destroy());
}
init().catch(console.error);
