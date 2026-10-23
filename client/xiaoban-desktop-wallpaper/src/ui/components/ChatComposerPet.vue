<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, shallowRef } from 'vue';
import type { PetDefinition } from '../../pets/model.ts';
import type PetActor from '../../pets/PetActor.vue';

const props = withDefaults(defineProps<{ size: 'sm' | 'lg'; paused?: boolean }>(), { paused: false });
const image = ref('');
const definition = shallowRef<PetDefinition | null>(null);
const actorComponent = shallowRef<typeof PetActor | null>(null);
const actor = ref<InstanceType<typeof PetActor> | null>(null);
const button = ref<HTMLButtonElement | null>(null);
const reacting = ref(false);
const response = ref('');
const responses = ['嘿嘿～', '被你发现啦！', '再摸摸头～'];
let responseIndex = 0;
let responseVersion = 0;
let resetTimer: ReturnType<typeof setTimeout> | undefined;
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
onBeforeUnmount(() => { mounted = false; ++responseVersion; clearTimeout(resetTimer); });

async function poke() {
  if (!definition.value) return;
  const version = ++responseVersion;
  clearTimeout(resetTimer);
  response.value = responses[responseIndex++ % responses.length];
  reacting.value = true;
  // Give immediate local feedback; load the heavy live animator only on first interaction.
  resetTimer = setTimeout(() => {
    if (version === responseVersion) { reacting.value = false; response.value = ''; }
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
      <component :is="actorComponent" v-if="reacting && actorComponent && definition" ref="actor" class="composer-pet-live" :definition="definition" :size="size === 'lg' ? 116 : 100" :paused="paused" :interactive="false" />
    </button>
    <span v-if="response" class="composer-pet-reply" role="status" data-testid="composer-pet-reply">{{ response }}</span>
  </div>
</template>

<style scoped>
.composer-pet{position:absolute;right:26px;top:-92px;width:100px;z-index:2;user-select:none}
.composer-pet.is-lg{right:32px;top:-106px;width:116px}
.composer-pet-touch{position:relative;display:block;width:100%;margin:0;padding:0;border:0;background:none;color:inherit;cursor:pointer;line-height:0;appearance:none;filter:drop-shadow(0 4px 6px rgba(0,0,0,.12));transition:filter .15s ease}
.composer-pet-touch:hover{filter:drop-shadow(0 5px 8px rgba(0,0,0,.16))}
.composer-pet-touch:focus-visible{outline:2px solid var(--ui-primary,#bd8c70);outline-offset:3px;border-radius:12px}
.composer-pet-picture{display:block;width:100%;height:auto;pointer-events:none}
.composer-pet-picture.is-hidden{visibility:hidden}
.composer-pet-live{position:absolute!important;left:0;top:0;pointer-events:none}
.composer-pet-live :deep(.pet-stage),.composer-pet-live :deep(.pet-art){pointer-events:none}
.composer-pet.is-reacting .composer-pet-touch{animation:pet-touch-response .4s ease-out}
.composer-pet-reply{position:absolute;bottom:calc(100% - 28px);right:calc(100% - 5px);padding:7px 11px;background:var(--ui-surface);border:1px solid var(--ui-border);border-radius:12px 12px 3px 12px;box-shadow:0 5px 15px rgba(0,0,0,.09);font-size:12px;line-height:1.3;color:var(--ui-text-primary);white-space:nowrap;pointer-events:none;animation:pet-reply-in .2s ease-out}
@keyframes pet-touch-response{0%,100%{transform:translateY(0) rotate(0)}30%{transform:translateY(-5px) rotate(-4deg)}65%{transform:translateY(-2px) rotate(3deg)}}
@keyframes pet-reply-in{from{opacity:0;transform:translateY(5px)}to{opacity:1;transform:translateY(0)}}
@media(prefers-reduced-motion:reduce){.composer-pet-touch,.composer-pet.is-reacting .composer-pet-touch,.composer-pet-reply{animation:none;transition:none}}
</style>
