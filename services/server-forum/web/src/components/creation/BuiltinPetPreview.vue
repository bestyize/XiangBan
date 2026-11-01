<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { BUILTIN_PETS } from '@/lib/builtinPets'
import type { PetView } from '@/lib/builtinPetEngine/pet'
import type { PetDefinition } from '@/lib/builtinPetEngine/pets/model'
const props = withDefaults(defineProps<{ id:string; playing?:boolean }>(), { playing:false })
const pet = computed(() => BUILTIN_PETS.find(item => item.id === props.id))
const host = ref<HTMLDivElement>()
const ready = ref(false)
let actor: PetView | undefined
let observer: IntersectionObserver | undefined
let request: AbortController | undefined
let generation = 0
let visible = true
const settings = () => ({ size:320, expression:'idle', paused:!props.playing || !visible, fps:30 as const, companionSkin:'pot' })
async function configure() {
  const current = ++generation
  request?.abort()
  actor?.destroy(); actor = undefined; ready.value = false
  if (!props.playing || !pet.value || !host.value) return
  const controller = request = new AbortController()
  try {
    const [runtime, response] = await Promise.all([
      import('@/lib/builtinPetEngine/pet'),
      fetch(`/builtin-pets/${pet.value.id}.json`, { signal:controller.signal }),
    ])
    if (!response.ok) throw new Error('伙伴暂不可用')
    const definition = await response.json() as PetDefinition
    if (current !== generation || !host.value) return
    actor = new runtime.PetView(host.value, settings(), undefined, definition)
    actor.reactions.setEnabled(false)
    host.value.querySelector('svg')?.setAttribute('viewBox', '0 -72 320 352')
    const art = host.value.querySelector('.pet-art')
    art?.setAttribute('role','button')
    art?.setAttribute('aria-label',`摸摸${pet.value?.name}，播放表情动作`)
    ready.value = true
  } catch (error) {
    if (!controller.signal.aborted) console.warn('伙伴动态预览加载失败', error)
  }
}
onMounted(() => {
  observer = new IntersectionObserver(entries => {
    visible = entries[0]?.isIntersecting ?? false
    actor?.update(settings())
  })
  if (host.value) observer.observe(host.value)
  void configure()
})
watch(() => [props.id, props.playing], () => { void configure() })
onBeforeUnmount(() => { generation++; request?.abort(); observer?.disconnect(); actor?.destroy() })
</script>
<template>
  <div class="builtin-pet-preview">
    <img v-if="pet && !ready" :src="pet.preview" :alt="`${pet.name}，官方内置伙伴`" width="640" height="704" loading="lazy" draggable="false">
    <div ref="host" class="pet-runtime" :class="{ ready }" @click="actor?.poke()" @pointermove="actor?.look($event.clientX,$event.clientY)" />
    <span v-if="!pet" class="missing">此伙伴暂不可用</span>
  </div>
</template>
<style scoped>
.builtin-pet-preview { position:relative; display:grid; place-items:center; width:100%; height:100%; min-height:120px; overflow:hidden; }
img,.pet-runtime { position:absolute; inset:0; display:block; width:100%; height:100%; }
img { object-fit:contain; user-select:none; }
.pet-runtime { visibility:hidden; }
.pet-runtime.ready { visibility:visible; cursor:pointer; }
.pet-runtime :deep(.pet-stage),.pet-runtime :deep(.pet-art),.pet-runtime :deep(svg) { display:block; width:100%!important; height:100%!important; }
.pet-runtime :deep(.pet-art:focus-visible) { outline:2px solid var(--brand); outline-offset:-3px; border-radius:12px; }
.missing { color:var(--ink-3); font-size:12px; }
</style>
