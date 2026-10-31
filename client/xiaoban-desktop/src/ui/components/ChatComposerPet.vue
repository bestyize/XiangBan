<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, shallowRef } from 'vue';
import type { PetDefinition } from '../../pets/model.ts';
import type PetActor from '../../pets/PetActor.vue';
import { desktop, invoke } from '../../bridge.ts';
import CompanionReplyCard from '../../companion-dialogue/CompanionReplyCard.vue';
import { previewReply, type CompanionReply } from '../../companion-dialogue/dialogue.ts';

const props = withDefaults(defineProps<{ size: 'sm' | 'lg'; paused?: boolean }>(), { paused: false });
const image = ref('');
const definition = shallowRef<PetDefinition | null>(null);
const actorComponent = shallowRef<typeof PetActor | null>(null);
const actor = ref<InstanceType<typeof PetActor> | null>(null);
const button = ref<HTMLButtonElement | null>(null);
const reacting = ref(false);
const response = ref<CompanionReply | null>(null);
let responseVersion = 0;
let resetTimer: ReturnType<typeof setTimeout> | undefined;
let dialogueTimer: ReturnType<typeof setTimeout> | undefined;
let mounted = true;

onMounted(async () => {
  try {
    const [{ OFFICIAL_THEMES }, { petImage }] = await Promise.all([
      import('../../pets/model.ts'),
      import('../../pets/render.ts'),
    ]);
    if (!mounted) return;
    const sunflower = OFFICIAL_THEMES.flatMap(theme => theme.pets).find(pet => pet.id === 'sunflower');
    if (sunflower) { definition.value = sunflower; image.value = petImage(sunflower); }
  } catch { /* The composer remains fully usable without optional pet assets. */ }
});
onBeforeUnmount(() => { mounted = false; ++responseVersion; clearTimeout(resetTimer); clearTimeout(dialogueTimer); });

async function poke() {
  if (!definition.value) return;
  const version = ++responseVersion;
  clearTimeout(resetTimer);
  clearTimeout(dialogueTimer);
  if (!desktop) response.value = previewReply();
  else dialogueTimer = setTimeout(() => {
    void invoke<CompanionReply>('companion_dialogue_click').then(value => {
      if (mounted && version === responseVersion && value) response.value = value;
    }).catch(() => { if (mounted && version === responseVersion) response.value = previewReply(); });
  }, 150);
  reacting.value = true;
  // Give immediate local feedback; load the heavy live animator only on first interaction.
  resetTimer = setTimeout(() => {
    if (version === responseVersion) reacting.value = false;
  }, 2400);
  if (!actorComponent.value) {
    try { actorComponent.value = (await import('../../pets/PetActor.vue')).default; }
    catch { return; /* The small bounce and bubble still work if loading fails. */ }
  }
  if (!mounted || version !== responseVersion || !reacting.value) return;
  await nextTick();
  if (!mounted || version !== responseVersion || !reacting.value) return;
  // The wrapping native button is the single keyboard target; don't tab into the SVG.
  button.value?.querySelector('.pet-art')?.setAttribute('tabindex', '-1');
  actor.value?.poke();
}
</script>

<template>
  <div v-if="image" class="composer-pet" :class="[`is-${size}`, { 'is-reacting': reacting }]">
    <button ref="button" type="button" class="composer-pet-touch" aria-label="摸摸小伴" title="点一下，摸摸小伴" data-testid="composer-pet" @click.stop="poke">
      <img :src="image" class="composer-pet-picture" :class="{ 'is-hidden': reacting && actorComponent }" alt="" aria-hidden="true" draggable="false" />
      <component :is="actorComponent" v-if="reacting && actorComponent && definition" ref="actor" class="composer-pet-live" :definition="definition" :size="size === 'lg' ? 58 : 50" :paused="paused" :interactive="false" />
    </button>
    <CompanionReplyCard v-if="response" class="composer-pet-reply" :initial="response" data-testid="composer-pet-reply" @close="response = null" />
  </div>
</template>

<style scoped>
.composer-pet{position:absolute;right:8px;top:-46px;width:50px;z-index:2;user-select:none}
.composer-pet.is-lg{right:10px;top:-53px;width:58px}
/* Match PetActor's square viewport so still/live swaps keep the same scale and anchor. */
.composer-pet-touch{position:relative;display:block;width:100%;aspect-ratio:1;margin:0;padding:0;border:0;background:none;color:inherit;cursor:pointer;line-height:0;appearance:none;filter:drop-shadow(0 4px 6px rgba(0,0,0,.12));transition:filter .15s ease}
.composer-pet-touch:hover{filter:drop-shadow(0 5px 8px rgba(0,0,0,.16))}
.composer-pet-touch:focus-visible{outline:2px solid var(--ui-primary,#bd8c70);outline-offset:3px;border-radius:12px}
.composer-pet-picture{display:block;width:100%;height:100%;object-fit:contain;pointer-events:none}
.composer-pet-picture.is-hidden{visibility:hidden}
.composer-pet-live{position:absolute!important;left:0;top:0;pointer-events:none}
.composer-pet-live :deep(.pet-stage),.composer-pet-live :deep(.pet-art){pointer-events:none}
.composer-pet.is-reacting .composer-pet-touch{animation:pet-touch-response .4s ease-out}
.composer-pet-reply{position:absolute;bottom:calc(100% - 14px);right:calc(100% - 5px);width:280px;max-width:calc(100vw - 44px);z-index:3;animation:pet-reply-in .2s ease-out}
@keyframes pet-touch-response{0%,100%{transform:translateY(0) rotate(0)}30%{transform:translateY(-5px) rotate(-4deg)}65%{transform:translateY(-2px) rotate(3deg)}}
@keyframes pet-reply-in{from{opacity:0;transform:translateY(5px)}to{opacity:1;transform:translateY(0)}}
@media(prefers-reduced-motion:reduce){.composer-pet-touch,.composer-pet.is-reacting .composer-pet-touch,.composer-pet-reply{animation:none;transition:none}}
</style>

