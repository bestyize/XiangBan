<script setup lang="ts">
import {
  computed,
  onActivated,
  onBeforeUnmount,
  onDeactivated,
  onMounted,
  ref,
} from "vue";
import { Animator } from "@/companion/animator";
import { Mind } from "@/companion/mind";
import { EXPRESSIONS, stillFrame, toSpec } from "@/companion/expressions";
import { SKIN_IDS, skinOf } from "@/companion/skins";
import {
  bodyTransform,
  earLight,
  earTransform,
  faceMarkup,
  faceOpacity,
  robotMarkup,
  shadowAttrs,
} from "@/companion/render";

const props = defineProps<{
  floating: boolean;
  cursor: { x: number; y: number } | null;
}>();
const emit = defineEmits<{ mood: [string]; interaction: [string] }>();
const host = ref<HTMLButtonElement | null>(null);
const position = ref({ x: 70, y: 65 });
/** The same plant shell the desktop app opens with. */
const skin = skinOf(SKIN_IDS[0]);
const style = computed(() => ({
  left: `${position.value.x}%`,
  top: `${position.value.y}%`,
}));
const art = robotMarkup(stillFrame("idle"), "site-xiaoban", "互动小伴花盆", skin.id);
let animator: Animator,
  mind: Mind,
  frame = 0,
  last = 0,
  lastMood = "",
  dragged = false;
let drag: {
  pointer: number;
  x: number;
  y: number;
  left: number;
  top: number;
} | null = null;
function animate(now: number) {
  frame = requestAnimationFrame(animate);
  if (!host.value || document.hidden) {
    last = 0;
    return;
  }
  if (last && now - last < 1000 / 30) return;
  const dt = last ? Math.min((now - last) / 1000, 0.05) : 1 / 30;
  last = now;
  if (props.cursor) {
    const rect = host.value.getBoundingClientRect();
    animator.lookAt = {
      x:
        (props.cursor.x - rect.left - rect.width / 2) /
        Math.max(rect.width, 180),
      y:
        (props.cursor.y - rect.top - rect.height / 2) /
        Math.max(rect.height, 160),
    };
    mind.activity();
  } else animator.lookAt = null;
  mind.tick(dt);
  const pose = animator.step(dt);
  const query = (selector: string) =>
    host.value!.querySelector<SVGElement>(selector);
  query('[data-part="body"]')?.setAttribute("transform", bodyTransform(pose));
  const face = query('[data-part="face"]');
  if (face) {
    face.innerHTML = faceMarkup(pose, "site-xiaoban");
    face.setAttribute("opacity", String(faceOpacity(pose)));
  }
  if (skin.earOuter) {
    for (const [side, index] of [
      ["L", -1],
      ["R", 1],
    ] as const) {
      const ear = query(`[data-part="ear${side}"]`);
      ear?.setAttribute("transform", earTransform(pose, index, skin.earPivot));
      ear
        ?.querySelector('[data-part="halo"]')
        ?.setAttribute("opacity", String(earLight(pose)));
    }
  }
  const shadow = query('[data-part="shadow"]');
  if (shadow) {
    const attrs = shadowAttrs(pose);
    shadow.setAttribute("rx", attrs.rx);
    shadow.setAttribute("opacity", attrs.opacity);
  }
  const mood = toSpec(mind.current()).expression;
  if (mood !== lastMood) {
    lastMood = mood;
    host.value.dataset.expression = mood;
    emit("mood", EXPRESSIONS[mood]?.label ?? "平静");
  }
}
function start() {
  if (animator && !frame) {
    last = 0;
    frame = requestAnimationFrame(animate);
  }
}
function stop() {
  cancelAnimationFrame(frame);
  frame = 0;
  last = 0;
}
onMounted(() => {
  animator = new Animator("idle", {
    reducedMotion: matchMedia("(prefers-reduced-motion: reduce)").matches,
  });
  mind = new Mind(animator, { autoSleep: false });
  start();
});
onActivated(start);
onDeactivated(stop);
onBeforeUnmount(stop);
function down(event: PointerEvent) {
  if (event.button !== 0) return;
  dragged = false;
  drag = {
    pointer: event.pointerId,
    x: event.clientX,
    y: event.clientY,
    left: position.value.x,
    top: position.value.y,
  };
  host.value!.setPointerCapture(event.pointerId);
}
function move(event: PointerEvent) {
  if (!drag || event.pointerId !== drag.pointer) return;
  const dx = event.clientX - drag.x,
    dy = event.clientY - drag.y;
  if (Math.hypot(dx, dy) > 4) dragged = true;
  if (!dragged) return;
  const scene = host.value!.parentElement!.getBoundingClientRect();
  position.value = {
    x: Math.max(18, Math.min(84, drag.left + (dx / scene.width) * 100)),
    y: Math.max(26, Math.min(72, drag.top + (dy / scene.height) * 100)),
  };
}
function up() {
  if (dragged) emit("interaction", "小伴搬到新位置了。");
  drag = null;
}
function poke() {
  if (dragged) {
    dragged = false;
    return;
  }
  mind?.poke();
  emit("interaction", "小伴收到你的戳戳啦。");
}
</script>
<template>
  <button
    ref="host"
    class="interactive-pet"
    :class="{ floating: props.floating }"
    :style="style"
    aria-label="戳戳小伴，也可拖动位置"
    @pointerdown="down"
    @pointermove="move"
    @pointerup="up"
    @pointercancel="up"
    @click="poke"
  >
    <span class="pet-art" v-html="art" />
  </button>
</template>
<style scoped>
.interactive-pet {
  position: absolute;
  z-index: 3;
  width: clamp(80px, 26cqw, 174px);
  aspect-ratio: 320 / 280;
  height: auto;
  transform: translate(-50%, -50%);
  touch-action: none;
  cursor: grab;
  filter: drop-shadow(0 8px 16px #78d2ff3d);
  transition:
    width 0.35s,
    height 0.35s;
  outline-offset: -12px;
}
.interactive-pet.floating {
  width: clamp(62px, 20cqw, 138px);
  height: auto;
  z-index: 6;
}
.interactive-pet:active {
  cursor: grabbing;
}
.pet-art {
  display: block;
  width: 100%;
  height: 100%;
  pointer-events: none;
}
.pet-art :deep(svg) {
  display: block;
  width: 100%;
  height: 100%;
}
@media (max-width: 500px) {
  .interactive-pet {
    width: clamp(76px, 26cqw, 110px);
    height: auto;
  }
  .interactive-pet.floating {
    width: clamp(60px, 20cqw, 88px);
    height: auto;
  }
}
</style>
