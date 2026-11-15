<script setup lang="ts">
import { ref, useId } from 'vue'

defineProps<{ count?: number | null }>()
const list = ref<HTMLElement | null>(null)
const titleId = useId()

function jump() {
  list.value?.focus({ preventScroll: true })
  list.value?.scrollIntoView({ block: 'start' })
}

defineExpose({ jump })
</script>

<template>
  <section ref="list" class="comment-list" tabindex="-1" :aria-labelledby="titleId">
    <h2 :id="titleId">评论 <span v-if="count != null">{{ count }}</span></h2>
    <slot />
  </section>
</template>

<style scoped>
.comment-list { margin-top: 16px; padding: 12px 0 16px; border-top: 1px solid var(--line); scroll-margin-top: calc(var(--header-h) + 12px); }
.comment-list h2 { margin: 0; font-size: 16px; font-weight: 650; }
.comment-list h2 span { margin-inline-start: 4px; color: var(--ink-3); font-size: 14px; font-weight: 400; }
</style>
