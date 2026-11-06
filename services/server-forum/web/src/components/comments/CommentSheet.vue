<script setup lang="ts">
import { computed, ref, watch } from 'vue'

// 手机上根节点是 Teleport，不能自动继承 class 等属性；只在桌面的行内版本上手动绑定。
defineOptions({ inheritAttrs: false })
const props = defineProps<{ open: boolean; inline?: boolean; title: string; id?: string }>()
const emit = defineEmits<{ close: [] }>()

// 软键盘弹出时 visualViewport 会变矮，把面板抬到键盘上方。
const inset = ref(0)
function syncInset() {
  const vv = window.visualViewport
  inset.value = vv ? Math.max(0, Math.round(window.innerHeight - vv.height - vv.offsetTop)) : 0
}
const floating = computed(() => props.open && !props.inline)
watch(floating, (on, _, cleanup) => {
  if (!on) return
  const root = document.documentElement, previous = root.style.overflow, vv = window.visualViewport
  root.style.overflow = 'hidden'
  vv?.addEventListener('resize', syncInset)
  vv?.addEventListener('scroll', syncInset)
  syncInset()
  cleanup(() => {
    root.style.overflow = previous
    vv?.removeEventListener('resize', syncInset)
    vv?.removeEventListener('scroll', syncInset)
    inset.value = 0
  })
})
</script>

<template>
  <!-- 桌面：原样放在页面里，只有内容；手机：底部弹出的面板，内容一直保持挂载，草稿和上传状态不会丢。 -->
  <section v-if="inline" v-bind="$attrs" :id="id" aria-label="评论"><slot /></section>
  <Teleport v-else to="body">
    <div v-if="open" class="sheet-backdrop" @click="emit('close')" />
    <section v-show="open" :id="id" class="comment-sheet" role="dialog" :aria-label="title" :style="{ bottom: `${inset}px` }" @keydown.esc="emit('close')">
      <div class="sheet-head">
        <strong>{{ title }}</strong>
        <button type="button" class="sheet-close" @click="emit('close')">收起</button>
      </div>
      <slot />
    </section>
  </Teleport>
</template>

<style scoped>
.sheet-backdrop {
  position: fixed;
  inset: 0;
  z-index: 39;
  background: rgba(0, 0, 0, 0.35);
}
.comment-sheet {
  position: fixed;
  z-index: 40;
  left: 0;
  right: 0;
  box-sizing: border-box;
  max-height: 80dvh;
  overflow: auto;
  padding: 12px 16px calc(12px + env(safe-area-inset-bottom));
  background: var(--surface);
  border-radius: 16px 16px 0 0;
  box-shadow: 0 -6px 24px rgba(0, 0, 0, 0.14);
}
.sheet-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  margin-bottom: 8px;
  font-size: var(--fs-sm);
}
.sheet-close {
  min-height: 36px;
  padding: 0 8px;
  border: 0;
  background: none;
  color: var(--ink-3);
  font-size: 13px;
}
</style>
