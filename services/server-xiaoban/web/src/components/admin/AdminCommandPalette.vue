<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import Icon, { type IconName } from '@/components/Icon.vue'

export interface CommandEntry {
  id: string
  label: string
  hint: string
  icon: IconName
  run: () => void
}

const props = defineProps<{ open: boolean; commands: CommandEntry[] }>()
const emit = defineEmits<{ close: [] }>()

const q = ref('')
const activeIndex = ref(0)
const inputEl = ref<HTMLInputElement>()

const filtered = computed(() => {
  const kw = q.value.trim().toLowerCase()
  if (!kw) return props.commands
  return props.commands.filter(
    (c) => c.label.toLowerCase().includes(kw) || c.hint.toLowerCase().includes(kw),
  )
})

watch(
  () => props.open,
  async (val) => {
    if (val) {
      q.value = ''
      activeIndex.value = 0
      await nextTick()
      inputEl.value?.focus()
    }
  },
)

watch(filtered, () => {
  activeIndex.value = 0
})

function execute(cmd?: CommandEntry) {
  if (!cmd) return
  emit('close')
  cmd.run()
}

function onKey(e: KeyboardEvent) {
  if (!props.open) return
  if (e.key === 'Escape') {
    e.preventDefault()
    emit('close')
  } else if (e.key === 'ArrowDown') {
    e.preventDefault()
    if (filtered.value.length) activeIndex.value = (activeIndex.value + 1) % filtered.value.length
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    if (filtered.value.length) activeIndex.value = (activeIndex.value - 1 + filtered.value.length) % filtered.value.length
  } else if (e.key === 'Enter') {
    e.preventDefault()
    execute(filtered.value[activeIndex.value])
  }
}

onMounted(() => document.addEventListener('keydown', onKey))
onBeforeUnmount(() => document.removeEventListener('keydown', onKey))
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="xb-backdrop is-center" @click.self="emit('close')">
      <div class="cmd-box" role="dialog" aria-label="快捷跳转">
        <div class="cmd-top">
          <Icon name="search" :size="15" />
          <input ref="inputEl" v-model="q" placeholder="快速跳转或操作…" />
          <span class="cmd-kbd">ESC</span>
        </div>
        <div class="cmd-list">
          <button
            v-for="(cmd, idx) in filtered"
            :key="cmd.id"
            type="button"
            class="xb-menu-item"
            :class="{ 'is-active': idx === activeIndex }"
            @mouseenter="activeIndex = idx"
            @click="execute(cmd)"
          >
            <span class="m-ico"><Icon :name="cmd.icon" :size="14" /></span>
            <span>{{ cmd.label }}</span>
            <span class="m-hint">{{ cmd.hint }}</span>
          </button>
          <div v-if="!filtered.length" class="cmd-empty">无匹配项</div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.cmd-box {
  width: min(460px, 100%);
  background: var(--ui-surface-raised);
  -webkit-backdrop-filter: blur(20px);
  backdrop-filter: blur(20px);
  border-radius: 16px;
  box-shadow: var(--ui-shadow-dialog);
  overflow: hidden;
  margin-top: -10vh;
}
.cmd-top {
  display: flex;
  align-items: center;
  gap: 9px;
  height: 46px;
  padding: 0 14px;
  border-bottom: 1px solid var(--ui-line);
  color: var(--ui-text-secondary);
}
.cmd-top input {
  flex: 1;
  border: 0;
  background: transparent;
  outline: none;
  font-size: 13.5px;
  color: var(--ui-text-primary);
}
.cmd-kbd {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: 20px;
  padding: 0 6px;
  border-radius: 5px;
  font-family: var(--ui-mono);
  font-size: 10.5px;
  color: var(--ui-text-tertiary);
  background: var(--ui-control);
}
.cmd-list {
  max-height: 320px;
  overflow-y: auto;
  padding: 6px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.cmd-empty {
  padding: 20px;
  text-align: center;
  color: var(--ui-text-tertiary);
  font-size: 12.5px;
}
</style>
