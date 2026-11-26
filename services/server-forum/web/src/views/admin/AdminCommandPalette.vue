<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { ADMIN_COMMANDS, adminSectionPath } from './adminNav'
import AdminIcon from './AdminIcon.vue'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: []; toggleTheme: [] }>()
const router = useRouter()
const query = ref('')
const inputRef = ref<HTMLInputElement | null>(null)
const activeIdx = ref(0)

const filtered = computed(() => {
  const kw = query.value.trim().toLowerCase()
  const list = ADMIN_COMMANDS.filter(item => !kw || item.keywords.includes(kw))
  return list
})

watch(
  () => props.open,
  async val => {
    if (!val) return
    query.value = ''
    activeIdx.value = 0
    await nextTick()
    inputRef.value?.focus()
  },
)

watch(query, () => {
  activeIdx.value = 0
})

function go(section: string) {
  emit('close')
  void router.push(adminSectionPath(section))
}

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    emit('close')
  } else if (e.key === 'ArrowDown') {
    e.preventDefault()
    if (filtered.value.length) activeIdx.value = (activeIdx.value + 1) % filtered.value.length
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    if (filtered.value.length) activeIdx.value = (activeIdx.value - 1 + filtered.value.length) % filtered.value.length
  } else if (e.key === 'Enter') {
    e.preventDefault()
    const target = filtered.value[activeIdx.value]
    if (target) go(target.section)
  }
}
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="adm-overlay adm-cmd-overlay" role="dialog" aria-modal="true" aria-label="快速跳转" @click.self="emit('close')">
      <div class="adm-cmd" @keydown="onKey">
        <div class="adm-cmd-input">
          <AdminIcon name="search" :size="16" />
          <input ref="inputRef" v-model="query" placeholder="输入模块名或路径（支持全部 23 个后台功能直达）…" />
          <span class="adm-kbd">ESC</span>
        </div>
        <div class="adm-cmd-list">
          <button
            v-for="(item, idx) in filtered"
            :key="item.section || 'overview'"
            type="button"
            class="adm-cmd-row"
            :class="{ 'is-on': idx === activeIdx }"
            @mouseenter="activeIdx = idx"
            @click="go(item.section)"
          >
            <span class="adm-cmd-title">{{ item.title }}</span>
            <span class="adm-cmd-meta">
              <span class="adm-tag">{{ item.group }}</span>
              <span class="adm-mono">{{ adminSectionPath(item.section) }}</span>
            </span>
          </button>
          <p v-if="!filtered.length" class="adm-empty" style="padding: 20px">没有匹配的模块</p>
        </div>
        <div class="adm-cmd-foot">
          <span>↑↓ 选择 · Enter 跳转</span>
          <button class="btn btn-ghost btn-sm" type="button" @click="emit('toggleTheme'); emit('close')">切换日夜外观</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
