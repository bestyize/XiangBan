import {
  desktop,
  defaults,
  invoke,
  listen,
  type Settings,
  type CursorSample,
} from "./bridge.ts";
import { PetView } from "./pet.ts";
import { listenPresence } from "./companion/presence.ts";
async function init() {
  const mode = new URLSearchParams(location.search).get("scene");
  if (!mode) {
    const { mountControl } = await import("./control.ts");
    await mountControl();
    return;
  }
  if (mode === "capture") {
    const { mountCapture } = await import("./capture/mount.ts");
    await mountCapture();
    return;
  }
  await import("./style.css");
  if (mode === "pet-instance") {
    const { mountPet } = await import("./pets/mount.ts");
    mountPet();
    return;
  }
  if (mode === "widget-settings") {
    const { mountWidgetSettings } = await import("./widgets/menu-mount.ts");
    await mountWidgetSettings();
    return;
  }
  if (mode === "widget") {
    const { mountWidget } = await import("./widgets/mount.ts");
    await mountWidget();
    return;
  }
  if (mode === "ritual") {
    const { mountRitual } = await import("./rituals/mount.ts");
    mountRitual();
    return;
  }
  if (mode === "effects") {
    const { mountEffects } = await import("./effects-scene.ts");
    await mountEffects();
    return;
  }
  if (mode === "desk") {
    const { mountDeskGame } = await import("./desk-scene.ts");
    await mountDeskGame();
    return;
  }
  if (mode === "games") {
    const { mountGameSpace } = await import("./games.ts");
    await mountGameSpace();
    return;
  }
  if (mode === "assistant") {
    const { mountAssistant } = await import("./assistant/mount.ts");
    await mountAssistant();
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
  document.body.className = "floating";
  document.querySelector<HTMLElement>("#app")!.innerHTML = '<section class="floating-stage"></section>';
  const pet = new PetView(document.querySelector<HTMLElement>(".floating-stage")!, config);
  let pointer: { x: number; y: number; dragged: boolean } | null = null,
    hovering: boolean | null = null;
  pet.host.addEventListener("pointerdown", (event) => {
    if (
      event.button !== 0 ||
      !pet.hit(event.clientX, event.clientY)
    )
      return;
    pointer = { x: event.clientX, y: event.clientY, dragged: false };
    pet.interact();
    pet.host.setPointerCapture(event.pointerId);
  });
  pet.host.addEventListener("pointermove", (event) => {
    if (
      pointer &&
      !pointer.dragged &&
      Math.hypot(event.clientX - pointer.x, event.clientY - pointer.y) > 5
    ) {
      pointer.dragged = true;
      pet.interact(8);
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
    if (hit !== hovering) {
      hovering = hit;
      invoke("set_hit_test", { hit }).catch(console.error);
    }
  });
  const apply = (settings: Settings) => {
    config = settings;
    document.body.dataset.theme = config.theme;
    document.body.dataset.paused = String(config.paused);
    pet.host.hidden = !config.companionEnabled;
    pet.update({ ...config, paused: config.paused || !config.companionEnabled });
    pet.reactions.setEnabled(config.companionLinks !== false);
    pet.reactions.setPaused(config.paused || !config.companionEnabled);
    pet.reactions.setEffect(config.effects.selected);
  };
  await listen<Settings>("settings-changed", apply);
  await listen("pet-poke", () => pet.poke());
  await listenPresence(listen, (beats) => pet.playBeats(beats), () => config.companionLinks !== false && config.companionEnabled);
  if (desktop) {
    apply(await invoke<Settings>("get_settings"));
    await invoke("scene_ready");
  } else apply(config);
  window.addEventListener("beforeunload", () => pet.destroy());
}
init().catch(console.error);

