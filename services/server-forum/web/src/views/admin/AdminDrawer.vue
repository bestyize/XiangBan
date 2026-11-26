<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import AdminIcon from './AdminIcon.vue'

const props = withDefaults(defineProps<{ open: boolean; title: string; wide?: boolean }>(), { wide: false })
const emit = defineEmits<{ close: [] }>()

function onKey(e: KeyboardEvent) {
  if (props.open && e.key === 'Escape') emit('close')
}
onMounted(() => window.addEventListener('keydown', onKey))
onUnmounted(() => window.removeEventListener('keydown', onKey))
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="adm-overlay is-drawer" role="dialog" aria-modal="true" :aria-label="title" @click.self="emit('close')">
      <div class="adm-drawer" :class="{ 'is-wide': wide }">
        <header class="adm-drawer-head">
          <h3>{{ title }}</h3>
          <button class="btn btn-ghost adm-icon-btn" type="button" aria-label="关闭" @click="emit('close')">
            <AdminIcon name="x" :size="16" />
          </button>
        </header>
        <div class="adm-drawer-body"><slot /></div>
        <footer v-if="$slots.footer" class="adm-drawer-foot"><slot name="footer" /></footer>
      </div>
    </div>
  </Teleport>
</template>
